import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as iam from 'aws-cdk-lib/aws-iam';

export class GreenThumbGuideStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const lambdaExecutionRole = new iam.Role(this, 'Lambda Execution Role', {
      assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
    });

    new cdk.CfnOutput(this, 'LambdaExecutionRole', { value: lambdaExecutionRole.roleArn });
  }
}
