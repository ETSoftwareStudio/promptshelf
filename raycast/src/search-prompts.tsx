import { Icon, List } from "@raycast/api";
import { useCachedPromise } from "@raycast/utils";
import { useEffect, useState } from "react";

import { PromptActions } from "./components/PromptActions";
import { PromptDetail } from "./components/PromptDetail";
import { InitView, InstallView } from "./components/SetupViews";
import { ShelfMissingError } from "./lib/binary";
import { pinsFirst, type PromptSummary } from "./lib/contract";
import { frontAppName, needsInit, showFailure } from "./lib/feedback";
import { pinColor } from "./lib/colors";
import { listLibraries, searchPrompts } from "./lib/shelf";

const ALL = "";

const KIND_ICONS: Record<string, Icon> = {
  template: Icon.Code,
  snippet: Icon.Paragraph,
  context: Icon.Person,
  persona: Icon.Person,
  workflow: Icon.List,
  draft: Icon.Tray,
};

export default function SearchPrompts() {
  const [query, setQuery] = useState("");
  const [library, setLibrary] = useState(ALL);
  const [selected, setSelected] = useState<string | null>(null);
  const [app, setApp] = useState<string>();
  useEffect(() => {
    void frontAppName().then(setApp);
  }, []);

  const libraries = useCachedPromise(listLibraries, [], { onError: () => undefined });
  const prompts = useCachedPromise((q: string, lib: string) => searchPrompts(q, lib || undefined), [query, library], {
    keepPreviousData: true,
    onError: (error) => {
      if (!(error instanceof ShelfMissingError) && !needsInit(error)) void showFailure(error);
    },
  });

  if (prompts.error instanceof ShelfMissingError || libraries.error instanceof ShelfMissingError) {
    return <InstallView />;
  }
  if (needsInit(prompts.error)) return <InitView />;

  const { pinned, rest } = pinsFirst(prompts.data ?? []);
  const manyLibraries = (libraries.data?.length ?? 0) > 1;

  const item = (p: PromptSummary) => (
    <List.Item
      key={p.id}
      id={p.id}
      title={p.title}
      icon={KIND_ICONS[p.kind] ?? Icon.Document}
      accessories={[
        ...(manyLibraries && library === ALL ? [{ tag: p.library }] : []),
        ...(p.pinned !== undefined
          ? [{ icon: { source: Icon.Tack, tintColor: pinColor }, text: String(p.pinned), tooltip: "Pinned" }]
          : []),
      ]}
      detail={selected === p.id ? <PromptDetail prompt={p} /> : <List.Item.Detail />}
      actions={<PromptActions prompt={p} app={app} onChange={prompts.revalidate} />}
    />
  );

  return (
    <List
      isLoading={prompts.isLoading}
      isShowingDetail
      filtering={false}
      throttle
      onSearchTextChange={setQuery}
      onSelectionChange={setSelected}
      searchBarPlaceholder="Search prompts"
      searchBarAccessory={
        <List.Dropdown tooltip="Library" value={library} onChange={setLibrary}>
          <List.Dropdown.Item title="All Libraries" value={ALL} />
          {(libraries.data ?? []).map((l) => (
            <List.Dropdown.Item key={l.name} title={l.name} value={l.name} />
          ))}
        </List.Dropdown>
      }
    >
      <List.EmptyView
        icon={Icon.MagnifyingGlass}
        title={query ? "No prompts match" : "No prompts yet"}
        description={query ? "Try other words, or All Libraries." : "Save one with Save Selection as Prompt."}
      />
      <List.Section title="Pinned">{pinned.map(item)}</List.Section>
      <List.Section title={query ? "Matches" : "Prompts"}>{rest.map(item)}</List.Section>
    </List>
  );
}
