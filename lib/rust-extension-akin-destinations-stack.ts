import * as cdk from "aws-cdk-lib";
import { RustExtension } from "cargo-lambda-cdk";
import { Construct } from "constructs";
import path = require("path");

export class RustExtensionAkinDestinationsStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const extensionLayer = new RustExtension(this, "RustExtension", {
      manifestPath: path.join(__dirname, "../extension/Cargo.toml"),
      binaryName: "extension",
      bundling: {
        architecture: cdk.aws_lambda.Architecture.ARM_64
      }
    });

    const destinationQueue = new cdk.aws_sqs.Queue(this, "DestinationQueue", {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      visibilityTimeout: cdk.Duration.seconds(30),
      retentionPeriod: cdk.Duration.days(1)
    });

    const receiverFunction = new cdk.aws_lambda_nodejs.NodejsFunction(
      this,
      "ReceiverFunction",
      {
        entry: path.join(__dirname, "./receiver-function.ts"),
        handler: "handler",
        runtime: cdk.aws_lambda.Runtime.NODEJS_18_X
      }
    );
    receiverFunction.addEventSourceMapping("ReadDestinationQueue", {
      eventSourceArn: destinationQueue.queueArn,
      batchSize: 1
    });
    /**
     * CDK will not do that for us.
     */
    destinationQueue.grantConsumeMessages(receiverFunction);

    const entryFunction = new cdk.aws_lambda_nodejs.NodejsFunction(
      this,
      "SampleFunction",
      {
        entry: path.join(__dirname, "./sample-function.ts"),
        handler: "handler",
        runtime: cdk.aws_lambda.Runtime.NODEJS_18_X,
        layers: [extensionLayer],
        architecture: cdk.aws_lambda.Architecture.ARM_64,
        environment: {
          DESTINATION_QUEUE_URL: destinationQueue.queueUrl
        }
      }
    );
    /**
     * Keep in mind that AWS Lambda Extensions share the IAM credentials with the AWS Lambda function.
     */
    destinationQueue.grantSendMessages(entryFunction);
  }
}
