import * as cdk from "aws-cdk-lib";
import { Template } from "aws-cdk-lib/assertions";
import * as CmsLambda from "../lib/cms_lambda-stack";

describe("Init Tests", () => {
  const app = new cdk.App();
  const stack = new CmsLambda.CmsLambdaStack(app, "Test-CmsLambdaStack");
  const template = Template.fromStack(stack);

  function logicalIdFromResource(resource: any) {
    try {
      const resKeys = Object.keys(resource);

      if (resKeys.length !== 1) {
        throw new Error("Resource is not unique");
      }
      const [logicalId] = resKeys;
      return logicalId;
    } catch (err) {
      console.log(resource);
      throw err;
    }
  }

  test("aws-iam-test", () => {
    let iam = template.findResources("AWS::IAM::Role", {});
  });

  test("aws-services-creation-test", () => {
    // Assert against Cloudformation Template
    let iam = template.findResources("AWS::IAM::Role", {});
    template.hasResource("AWS::S3::Bucket", {});

    template.hasResource("AWS::ApiGateway::RestApi", {
      Properties: { Name: "cms-api-gateway" },
    });

    template.hasResource("AWS::Lambda::Function", {
      Properties: { FunctionName: "s3-presigned" },
    });
    // template.hasResource("AWS::Lambda::Function", {
    //   Properties: { FunctionName: "s3-multipart" },
    // });
  });

  test("aws-apigateway-test", () => {
    // Currently testing only the resource lambda
    const resourcesPath = template.findResources("AWS::ApiGateway::Resource", {
      Properties: {
        PathPart: "resources",
      },
    });
    const resourcesGetLambda = template.findResources("AWS::Lambda::Function", {
      Properties: {
        TracingConfig: {
          Mode: "Active",
        },
      },
    });
    const resourcesPathId = logicalIdFromResource(resourcesPath);
    const resourcesGetLambdaId = logicalIdFromResource(resourcesGetLambda);

    const restApiId = logicalIdFromResource(
      template.findResources("AWS::ApiGateway::RestApi", {
        Properties: {
          Name: "cms-api-gateway",
        },
      }),
    );

    // Assertions
    template.hasResourceProperties("AWS::ApiGateway::Method", {
      HttpMethod: "GET",
      ResourceId: {
        Ref: resourcesPathId,
      },
      RestApiId: {
        Ref: restApiId,
      },
      Integration: {
        Uri: {
          "Fn::Join": [
            "",
            [
              "arn:",
              { Ref: "AWS::Partition" },
              ":apigateway:",
              { Ref: "AWS::Region" },
              ":lambda:path/2015-03-31/functions/",
              {
                "Fn::GetAtt": [resourcesGetLambdaId, "Arn"],
              },
              "/invocations",
            ],
          ],
        },
      },
      // Added this is in the Apigw for mapping reponses to Integration requests
      MethodResponses: [
        {
          StatusCode: "200",
          ResponseParameters: { "method.response.header.Content-Type": true },
          ResponseModels: { "application/json": "Empty" },
        },
      ],
    });

    template.hasResourceProperties("AWS::ApiGateway::Method", {
      HttpMethod: "PUT",
      ResourceId: {
        Ref: resourcesPathId,
      },
      RestApiId: {
        Ref: restApiId,
      },
    });

    template.hasResourceProperties("AWS::ApiGateway::Method", {
      HttpMethod: "DELETE",
      ResourceId: {
        Ref: resourcesPathId,
      },
      RestApiId: {
        Ref: restApiId,
      },
    });
  });

  test("aws-s3-lambda-intg-test", () => {
    let s3Id = logicalIdFromResource(
      template.findResources("AWS::S3::Bucket", {}),
    );

    template.hasResourceProperties("AWS::Lambda::Function", {
      Architectures: ["x86_64"],
      Runtime: "provided.al2",
      Environment: {
        Variables: {
          BUCKET_NAME: {
            Ref: s3Id,
          },
        },
      },
    });
  });
});
