# Deployment of htsget-lambda

The [htsget-lambda] crate is a cloud-based implementation of [htsget-rs]. It uses AWS Lambda as the ticket server, and AWS S3 as the data block server.

This is an example that deploys [htsget-lambda] using [aws-cdk]. It is deployed as an AWS HTTP [API Gateway Lambda proxy
integration][aws-api-gateway]. The stack uses [RustFunction][rust-function] in order to integrate [htsget-lambda]
with API Gateway. It also has the option to use a [JWT authorizer][jwt-authorizer] with [AWS Cognito][aws-cognito] as the issuer. The
JWT authorizer automatically verifies JWT tokens issued by Cognito. Routing for the server is done using [AWS Route 53][route-53].

## Configuration

The CDK code in this directory constructs a CDK app from [`HtsgetLambdaStack`][htsget-lambda-stack], and uses a settings file under [`bin/settings.ts`][htsget-settings]. To configure the deployment, change these settings in
[`bin/settings.ts`][htsget-settings]:

#### HtsgetSettings
These are general settings for the CDK deployment.

| Name                                                     | Description                                                                                                                                                                                                                                       | Type                                              |
|----------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------|
| <span id="config">`config`</span>                        | The location of the htsget-rs server config. This must be specified. This config file configures the htsget-rs server. See [htsget-config] for a list of available server configuration options.                                                  | `string`                                          | 
| <span id="domain">`domain`</span>                        | The domain name for the Route53 Hosted Zone that the htsget-rs server will be under. This must be specified. A hosted zone with this name will either be looked up or created depending on the value of [`lookupHostedZone?`](#lookupHostedZone). | `string`                                          |
| <span id="authorizer">`authorizer`</span>                | Deployment options related to the authorizer. Note that this option allows specifying an AWS [JWT authorizer][jwt-authorizer]. The JWT authorizer automatically verifies tokens issued by a Cognito user pool.                                    | [`HtsgetJwtAuthSettings`](#htsgetjwtauthsettings) |
| <span id="subDomain">`subDomain?`</span>                 | The domain name prefix to use for the htsget-rs server. Together with the [`domain`](#domain), this specifies url that the htsget-rs server will be reachable under. Defaults to `"htsget"`.                                                      | `string`                                          |
| <span id="s3BucketResources">`s3BucketResources?`</span> | The resources that are affected by the bucket policy with actions: `["s3:List*", "s3:Get*"]`. If this is not specified, it defaults to `["arn:aws:s3:::*"]`. This affects which buckets are allowed to be accessed with the policy.               | `string[]`                                        |
| <span id="lookupHostedZone">`lookupHostedZone?`</span>   | Whether to lookup the hosted zone with the domain name. Defaults to `true`. If `true`, attempts to lookup an existing hosted zone using the domain name. Set this to `false` if you want to create a new hosted zone with the domain name.        | `boolean`                                         |

#### HtsgetJwtAuthSettings
These settings are used to determine if the htsget API gateway endpoint is configured to have a JWT authorizer or not.

| Name                                              | Description                                                                                                                                               | Type       |
|---------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------|------------|
| <span id="public">`public`</span>                 | Whether this deployment is public. If this is `true` then no authorizer is present on the API gateway and the options below have no effect.               | `boolean`  |
| <span id="jwtAudience">`jwtAudience?`</span>      | A list of the intended recipients of the JWT. A valid JWT must provide an aud that matches at least one entry in this list.                               | `string[]` | 
| <span id="cogUserPoolId?">`cogUserPoolId?`</span> | The cognito user pool id for the authorizer. If this is not set, then a new user pool is created. No user pool is created if [`public`](#public) is true. | `string`   |

The [`HtsgetSettings`](#htsgetsettings) are passed into [`HtsgetLambdaStack`][htsget-lambda-stack] in order to change the deployment config. An example of a public instance deployment
can be found under [`bin/htsget-lambda.ts`][htsget-lambda-bin]. This uses the [`config/public_umccr.toml`][public-umccr-toml] server config. See [htsget-config] for a list of available server configuration options.

## Deploying

### Prerequisites

1. [aws-cli] should be installed and authenticated in the shell.
1. Node.js and [npm] should be installed.
1. [Rust][rust] should be installed.
1. [Zig][zig] should be installed

After installing the basic dependencies, complete the following steps:

1. Define CDK\_DEFAULT\_* env variables (if not defined already). You must be authenticated with your AWS cloud to run this step.
1. Add the arm cross-compilation target to rust.
1. Install [cargo-lambda], as it is used to compile artifacts that are uploaded to aws lambda.
1. Define which configuration to use for htsget-rs on `cdk.json` as stated in aforementioned configuration section. 

Below is a summary of commands to run in this directory:

```sh
``export CDK_DEFAULT_ACCOUNT=`aws sts get-caller-identity --query Account --output text`
export CDK_DEFAULT_REGION=`aws configure get region```
rustup target add aarch64-unknown-linux-gnu
cargo install cargo-lambda
npm install
```

### Deploy to AWS

CDK should be bootstrapped once, if this hasn't been done before.

```sh
npx cdk bootstrap
```

In order to deploy, check that the stack synthesizes correctly and then deploy.

```sh
npx cdk synth
npx cdk deploy
```

### Testing the endpoint

When the deployment is finished, the htsget endpoint can be tested by querying it. If a JWT authorizer is configured,
a valid JWT token must be obtained in order to access the endpoint. This token should be obtained from AWS Cognito using
the configured audience parameters. Then `curl` can be used to query the endpoint:

```sh
curl -H "Authorization: <JWT Token>" "https://<htsget_domain>/reads/service-info"
```

With a possible output:

```json
{
  "id": "",
  "name": "",
  "version": "",
  "organization": {
    "name": "",
    "url": ""
  },
  "type": {
    "group": "",
    "artifact": "",
    "version": ""
  },
  "htsget": {
    "datatype": "reads",
    "formats": ["BAM", "CRAM"],
    "fieldsParametersEffective": false,
    "TagsParametersEffective": false
  },
  "contactUrl": "",
  "documentationUrl": "",
  "createdAt": "",
  "UpdatedAt": "",
  "environment": ""
}
```

[awscurl]: https://github.com/okigan/awscurl

### Local testing

The [Lambda][htsget-lambda] function can also be run locally using [cargo-lambda]. From the root project directory, execute the following command.

```sh
cargo lambda watch
```

Then in a **separate terminal session** run.

```sh
cargo lambda invoke htsget-lambda --data-file data/events/event_get.json
```

Examples of different Lambda events are located in the [`data/events`][data-events] directory.

## Docker

There are example deployments using Docker under the [examples] directory. These include a [`LocalStorage`][local] deployment
and a [MinIO][minio] deployment.

[local]: examples/local_storage/README.md
[examples]: examples
[minio]: examples/minio/README.md
[htsget-lambda-bin]: bin/htsget-lambda.ts
[htsget-lambda-stack]: lib/htsget-lambda-stack.ts
[htsget-settings]: bin/settings.ts
[public-umccr-toml]: config/public_umccr.toml
[htsget-lambda]: ../htsget-lambda
[cargo-lambda]: https://github.com/cargo-lambda/cargo-lambda
[data-events]: ../data/events
[htsget-rs]: https://github.com/umccr/htsget-rs
[htsget-lambda]: ../htsget-lambda
[htsget-config]: ../htsget-config
[config]: config
[aws-cdk]: https://docs.aws.amazon.com/cdk/v2/guide/getting_started.html
[cdk-context]: https://docs.aws.amazon.com/cdk/v2/guide/context.html
[cdk-lookup-value]: https://docs.aws.amazon.com/cdk/api/v2/docs/aws-cdk-lib.aws_ssm.StringParameter.html#static-valuewbrfromwbrlookupscope-parametername
[cdk-json]: cdk.json
[aws-ssm]: https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html
[aws-api-gateway]: https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html
[aws-cognito]: https://docs.aws.amazon.com/cognito/latest/developerguide/cognito-user-identity-pools.html
[jwt-authorizer]: https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-jwt-authorizer.html
[jwt-audience]: https://docs.aws.amazon.com/apigatewayv2/latest/api-reference/apis-apiid-authorizers-authorizerid.html#apis-apiid-authorizers-authorizerid-model-jwtconfiguration
[route-53]: https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/Welcome.html
[rust-function]: https://www.npmjs.com/package/rust.aws-cdk-lambda
[aws-cdk]: https://docs.aws.amazon.com/cdk/v2/guide/getting_started.html
[aws-cli]: https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html
[npm]: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
[rust]: https://www.rust-lang.org/tools/install
[zig]: https://ziglang.org/
[zig-getting-started]: https://ziglang.org/learn/getting-started/
