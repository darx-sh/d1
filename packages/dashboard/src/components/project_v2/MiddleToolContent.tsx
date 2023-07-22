import CodeMirror from "@uiw/react-codemirror";
import { githubLight } from "@uiw/codemirror-theme-github";
import { javascript } from "@codemirror/lang-javascript";
import { EditorView } from "@codemirror/view";
import { useProjectState, useProjectDispatch } from "./ProjectContext";
import React from "react";

const myTheme = EditorView.theme({
  "&": {
    fontSize: "1rem",
    lineHeight: "1.5rem",
  },
});

export default function MiddleToolContent() {
  const projectState = useProjectState();
  const projectDispatch = useProjectDispatch();

  const renderTabContent = (tabIdx: number) => {
    const tab = projectState!.tabs[tabIdx]!;
    switch (tab.type) {
      case "JsEditor": {
        const code = projectState!.directory.codes[tab.codeIdx];
        return (
          <CodeMirror
            value={code!.content}
            theme={githubLight}
            extensions={[javascript(), myTheme]}
            onChange={(value, viewUpdate) => {
              const t = tab as { type: "JsEditor"; codeIdx: number };
              projectDispatch!({
                type: "UpdateJsFile",
                codeIdx: t.codeIdx,
                content: value,
              });
            }}
          ></CodeMirror>
        );
      }
      case "Database": {
        return <div>Database</div>;
      }
    }
  };

  return projectState!.curOpenTabIdx === null ? (
    <></>
  ) : (
    renderTabContent(projectState!.curOpenTabIdx)
  );
}
