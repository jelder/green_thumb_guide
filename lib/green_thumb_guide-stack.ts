import { Stack, StackProps, Duration } from "aws-cdk-lib";
import { Construct } from "constructs";
import { RustFunction } from "rust.aws-cdk-lambda";
import * as logs from "aws-cdk-lib/aws-logs";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as apig from "aws-cdk-lib/aws-apigatewayv2";
import * as apigIntegrations from "aws-cdk-lib/aws-apigatewayv2-integrations";
import * as route53 from "aws-cdk-lib/aws-route53";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import * as targets from "aws-cdk-lib/aws-route53-targets";

type Props = StackProps & {
  domain: string;
  subdomain: string;
};

export class GreenThumbGuideStack extends Stack {
  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id, props);

    const hardinessZoneFunction = new RustFunction(
      this,
      "HardinessZoneFunction",
      {
        directory: "./usda_hardiness_zone/",
        logGroup: new logs.LogGroup(this, "HardinessZoneFunctionLogGroup"),
        timeout: Duration.seconds(3),
        layers: [
          new lambda.LayerVersion(this, "HardinessDatabaseLayer", {
            code: lambda.Code.fromAsset("./usda_hardiness_zone/database.zip"),
          }),
        ],
        setupLogging: true,
        reservedConcurrentExecutions: 10,
      },
    );

    const httpApi = new apig.HttpApi(this, "GreenThumbGuideHttpApi", {
      defaultAuthorizer: new apig.HttpNoneAuthorizer(),
    });

    httpApi.addRoutes({
      path: "/{proxy+}",
      methods: [apig.HttpMethod.ANY],
      integration: new apigIntegrations.HttpLambdaIntegration(
        "LambdaIntegration",
        hardinessZoneFunction,
      ),
    });

    const hostedZone = route53.HostedZone.fromLookup(this, "HostedZone", {
      domainName: props.domain,
    });

    const fqdn = `${props.subdomain}.${props.domain}`;
    const domainName = new apig.DomainName(this, "DomainName", {
      domainName: fqdn,
      certificate: new acm.Certificate(this, "Certificate", {
        domainName: fqdn,
        validation: acm.CertificateValidation.fromDns(hostedZone),
      }),
    });

    new apig.ApiMapping(this, "HttpApiMapping", {
      api: httpApi,
      domainName,
    });

    new route53.ARecord(this, "CustomDomainAliasRecord", {
      zone: hostedZone,
      target: route53.RecordTarget.fromAlias(
        new targets.ApiGatewayv2DomainProperties(
          domainName.regionalDomainName,
          domainName.regionalHostedZoneId,
        ),
      ),
      recordName: props.subdomain,
    });
  }
}
