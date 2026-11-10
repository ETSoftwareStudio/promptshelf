import { Detail, Form, type LaunchProps } from "@raycast/api";
import { usePromise } from "@raycast/utils";

import { FillForm } from "./components/FillForm";
import { InstallView } from "./components/SetupViews";
import { ShelfMissingError } from "./lib/binary";
import { deliver, showFailure } from "./lib/feedback";
import { fillPrompt, showPrompt } from "./lib/shelf";

/**
 * Fill: a Quicklink target. Pastes at once when every blank is filled;
 * otherwise opens the Fill form.
 */
async function start(id: string) {
  const fill = await fillPrompt(id);
  if (fill.complete) {
    await deliver(fill.text, "paste");
    return undefined;
  }
  const show = await showPrompt(id);
  return { fill, show };
}

export default function Fill(props: LaunchProps<{ arguments: Arguments.Fill }>) {
  const id = props.arguments.id.trim();
  const { data, isLoading, error } = usePromise(start, [id], {
    onError: (e) => {
      if (!(e instanceof ShelfMissingError)) void showFailure(e);
    },
  });

  if (error instanceof ShelfMissingError) return <InstallView />;
  if (error) return <Detail markdown={`**Could not fill ${id}.**\n\n${error.message}`} />;
  if (isLoading || !data) return <Form isLoading />;
  return <FillForm title={data.show.title} blanks={data.show.blanks} fill={data.fill} />;
}
