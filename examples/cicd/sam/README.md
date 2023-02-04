## Actix Web on Lambda & API Gateway with SAM

Deploy your application with [AWS SAM](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/what-is-sam.html), using [SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html). Have a look at a sample role you will want to create and assign to your user.
```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "NotAction": [
                "iam:*",
                "organizations:*",
                "account:*"
            ],
            "Resource": "*"
        },
        {
            "Effect": "Allow",
            "Action": [
                "iam:CreateServiceLinkedRole",
                "iam:DeleteServiceLinkedRole",
                "iam:ListRoles",
                "iam:ListPolicies",
                "organizations:DescribeOrganization",
                "account:ListRegions"
            ],
            "Resource": "*"
        },
        {
            "Effect": "Allow",
            "Action": "iam:*",
            "Resource": "arn:aws:iam::37********43:role/microservice-*-MicroServiceLambdaRole*"
        }
    ]
}
```
Its core has been taken from AWS-managed PowerUser. We are just assigning some additional IAM perms wih regard to concrete lambda resource.
In case you are operating on a limited scope of resource, you will want to cap those permissions, adhering to the principle of least privilege.

Put Makefile (see `./Makefile`) into your project's root - at the same level where your SAM template (see `./template.yaml`) is located and hit:
```
make build
make package
make deploy
```
You can use - as an option - GitHub Actions to automate the deployment. Add the builda and deploy jobs to your project's `.github/workflows` directory (see `./sam-deploy.yaml` sample jobs configuration).

Do not forget to clean up with `make destroy` to avoid unwanted costs. Alternatively, you can delete the stack via AWS Console (CloudFormation service).
