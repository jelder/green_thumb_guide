import { Stack, StackProps, CfnOutput, Duration } from "aws-cdk-lib";
import { Construct } from "constructs";
import { RustFunction } from 'rust.aws-cdk-lambda';
import * as logs from 'aws-cdk-lib/aws-logs';
import * as aws_lambda from 'aws-cdk-lib/aws-lambda';
import * as apig from 'aws-cdk-lib/aws-apigatewayv2';
import * as apigIntegrations from 'aws-cdk-lib/aws-apigatewayv2-integrations';
import * as route53 from 'aws-cdk-lib/aws-route53';
import * as acm from 'aws-cdk-lib/aws-certificatemanager';
import * as targets from 'aws-cdk-lib/aws-route53-targets';

type Props = StackProps & {
  domain: string,
  subdomain: string,
}

export class GreenThumbGuideStack extends Stack {
  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id, props);

    const fqdn = `${props.subdomain}.${props.domain}`;

    const hardinessZoneFunction = new RustFunction(this, "USDAHardinessZoneFunction", {
      directory: "../usda_hardiness_zone/",
      logGroup: new logs.LogGroup(this, 'USDAHardinessZoneFunctionLogGroup'),
      timeout: Duration.seconds(3),
    })

    const hardinessZoneFunctionUrl = new aws_lambda.FunctionUrl(this, "USDAHardinessZoneFunctionUrl", {
      function: hardinessZoneFunction,
      authType: aws_lambda.FunctionUrlAuthType.NONE,
    })
    new CfnOutput(this, 'url', { value: hardinessZoneFunctionUrl.url });

    const httpApi = new apig.HttpApi(this, 'GreenThumbGuideHttpApi');

    httpApi.addRoutes({
      path: '/hardiness_zone',
      methods: [apig.HttpMethod.GET, apig.HttpMethod.POST],
      integration: new apigIntegrations.HttpLambdaIntegration('LambdaIntegration', hardinessZoneFunction),
    });

    const hostedZone = route53.HostedZone.fromLookup(this, 'HostedZone', {
      domainName: props.domain,
    });

    const certificate = new acm.Certificate(this, 'Certificate', {
      domainName: fqdn,
      validation: acm.CertificateValidation.fromDns(hostedZone),
    });

    const domainName = new apig.DomainName(this, 'DomainName', {
      domainName: fqdn,
      certificate,
    });

    new route53.ARecord(this, 'CustomDomainAliasRecord', {
      zone: hostedZone,
      target: route53.RecordTarget.fromAlias(new targets.ApiGatewayv2DomainProperties(domainName.regionalDomainName, domainName.regionalHostedZoneId)),
      recordName: props.subdomain,
    });
  }
}
