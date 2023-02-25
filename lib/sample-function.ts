import { asyncResult } from "@expo/results";
import { Context } from "aws-lambda";
import got from "got";

const handlerFn = async () => {
  if (Math.random() > 0.5) {
    throw new Error("Boom error!");
  }

  return JSON.stringify({ message: "From the AWS Lambda!" });
};

type AsyncHandler<Event, Response> = (
  event: Event,
  context: Context
) => Promise<Response>;

const middleware =
  <Event, Response>(handler: AsyncHandler<Event, Response>) =>
  async (event: Event, context: Context) => {
    const result = await asyncResult(handler(event, context));
    /**
     * Safety net. If we are running out of time, we should not make any more requests.
     */
    if (context.getRemainingTimeInMillis() < 1_000) {
      return result.enforceValue();
    }

    /**
     * We cannot use a fire-and-forget style of request here since AWS Lambda runtime might freeze this handler.
     */
    const sendPayloadResult = await asyncResult(
      got.post("http:127.0.0.1:8080", {
        json: {
          event: event,
          result: result.ok ? result.value : result.reason.message
        }
      })
    );
    if (!sendPayloadResult.ok) {
      console.log("error while sending the payload", sendPayloadResult.reason);
      throw new Error("Failure");
    }

    return result.enforceValue();
  };

export const handler = middleware(handlerFn);
