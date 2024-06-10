import { aws_apigateway, aws_s3 } from "aws-cdk-lib";
import {
  LambdaIntegration,
  LambdaRestApi,
  RestApi,
} from "aws-cdk-lib/aws-apigateway";
import { IFunction } from "aws-cdk-lib/aws-lambda";
import { Construct } from "constructs";

interface ApiGatewayStackProps {
  S3PresignedLambda: IFunction;
  MultiPartLambda: IFunction;
}

interface ResourceType {
  name: string;
  methods: string[];
  child?: ResourceType;
}

export class ApiGatewayStack extends Construct {
  constructor(scope: Construct, id: string, props: ApiGatewayStackProps) {
    super(scope, id);

    this.addResource(id, props);
  }

  addResource(
    serviceName: string,
    { S3PresignedLambda, MultiPartLambda }: ApiGatewayStackProps,
  ) {
    const apigw = new aws_apigateway.RestApi(this, `${serviceName}`);

    this.createEndPoints(S3PresignedLambda, apigw, {
      name: "presigned",
      methods: ["GET", "PUT", "DELETE"],
    });

    this.createEndPoints(MultiPartLambda, apigw, {
      name: "multipart",
      methods: ["POST"],
    });
  }

  createEndPoints(
    handler: IFunction,
    apigw: RestApi,
    { name, methods, child }: ResourceType,
  ) {
    const lambdaIntegration = new LambdaIntegration(handler, {
      // Map method request data to integration request parameters
      // (https://docs.aws.amazon.com/apigateway/latest/developerguide/request-response-data-mappings.html#mapping-response-parameters)
      passthroughBehavior: aws_apigateway.PassthroughBehavior.WHEN_NO_TEMPLATES,
      requestParameters: {
        "integration.request.header.Content-Type":
          "'application/x-www-form-urlencoded'",
      },
      integrationResponses: [
        {
          statusCode: "200",
          responseParameters: {
            "method.response.header.Content-Type":
              "integration.response.header.Content-Type",
          },
        },
      ],
    });

    const rootResource = apigw.root.addResource(name);
    methods.map((item) => {
      rootResource.addMethod(item, lambdaIntegration, {
        // For proper response `Content-Type` header from the lambdas
        methodResponses: [
          {
            statusCode: "200",
            responseModels: {
              "application/json": aws_apigateway.Model.EMPTY_MODEL,
            },
            responseParameters: {
              // TODO: Test this
              // ensures the handler returns this header
              "method.response.header.Content-Type": true,
            },
          },
        ],
      });
    });

    // Currently no child methods in the lambda
    if (child) {
      const childResource = rootResource.addResource(child.name);
      child.methods.map((item) => {
        childResource.addMethod(item, lambdaIntegration);
      });
    }
  }
}
