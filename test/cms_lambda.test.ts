import * as cdk from "aws-cdk-lib";
import { Template } from "aws-cdk-lib/assertions";
import * as CmsLambda from "../lib/cms_lambda-stack";

describe("Init Tests", () => {
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

  test("aws-services-created", () => {
    const app = new cdk.App();
    const stack = new CmsLambda.CmsLambdaStack(app, "Test-CmsLambdaStack");

    const template = Template.fromStack(stack);

    // Assert against Cloudformation Template
    template.hasResource("AWS::S3::Bucket", {});

    template.hasResource("AWS::ApiGateway::RestApi", {
      Properties: { Name: "cms-api-gateway" },
    });

    template.hasResource("AWS::Lambda::Function", {
      Properties: { FunctionName: "s3-resources" },
    });
    template.hasResource("AWS::Lambda::Function", {
      Properties: { FunctionName: "s3-multipart" },
    });
  });

  test("aws-apigateway-test", () => {
    const app = new cdk.App();
    const stack = new CmsLambda.CmsLambdaStack(app, "Test-CmsLambdaStack");

    const template = Template.fromStack(stack);

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
});
