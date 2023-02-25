#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { RustExtensionAkinDestinationsStack } from "../lib/rust-extension-akin-destinations-stack";

const app = new cdk.App();
new RustExtensionAkinDestinationsStack(
  app,
  "RustExtensionAkinDestinationsStack",
  {
    synthesizer: new cdk.DefaultStackSynthesizer({
      qualifier: "rext"
    })
  }
);
