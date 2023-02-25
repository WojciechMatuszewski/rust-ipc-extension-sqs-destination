# A sample AWS Lambda extension written in Rust

Inspired by [this post](https://danwakeem.medium.com/extension-review-funky-async-extension-8e5021343d00) and the inability to use [AWS Lambda destinations](https://aws.amazon.com/blogs/compute/introducing-aws-lambda-destinations/) when dealing with _synchronous_ invocations (keep in mind that SQS -> Lambda is also synchronous from the ESM point of view).

## How this stuff works

1. The AWS Lambda function sends it's payload and the result of the invocation to a local HTTP endpoint spun up by AWS Lambda extension.

2. The AWS Lambda extension sends that payload to SQS. The message is picked up by another AWS Lambda.

### Caveats

1. Since the AWS Lambda extension API (and the Telemetry API) cannot read the event and response of the function, I cannot think of any way of providing those to the AWS extension other than sending that HTTP request.

    - This sucks as we are blocking the execution of the function. The request will be fast, but we are still waiting for it to finish. We cannot do "fire and forget" here as AWS Lambda freezes the execution environment. If we request would fail, we might fail in the **next** invalidation of a AWS Lambda function.

## Learnings

- It is impossible to use `--no-rollback` option with some resources.

  - The CFN will not let you deploy if a resource is _immutable_, like the AWS Lambda layer version.

- Rust has a pretty mature ecosystem of web-servers. I was able to create a basic HTTP server in no time.

  - I'm pretty impressed what people can do with macros. Such a powerful language feature!
