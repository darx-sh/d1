import React from "react";
import CodeMirror from "@uiw/react-codemirror";
import { githubLight } from "@uiw/codemirror-theme-github";
import { javascript } from "@codemirror/lang-javascript";
import { EditorView } from "@codemirror/view";
import { useProjectState, useProjectDispatch } from "./ProjectContext";
import HttpEndpoints from "~/components/project/HttpEndpoints";
import Database from "~/components/project/database/Database";

const myTheme = EditorView.theme({
  "&": {
    fontSize: "1rem",
    lineHeight: "1.5rem",
    maxHeight: "670px",
  },
  "&.cm-focused": {
    outline: "none",
  },
  ".cm-scroller": { overflow: "auto" },
});

export default function ToolContent() {
  const projectState = useProjectState()!;
  const projectDispatch = useProjectDispatch()!;

  const renderTabContent = (tabIdx: number) => {
    const tab = projectState.tabs[tabIdx]!;
    switch (tab.type) {
      case "JsEditor": {
        const code = projectState.directory.codes[tab.codeIdx];
        return (
          <div className="flex h-full justify-end space-x-2 bg-white">
            <div className="flex-1">
              <CodeMirror
                value={code!.content}
                theme={githubLight}
                extensions={[javascript(), myTheme, EditorView.lineWrapping]}
                onChange={(value, viewUpdate) => {
                  const t = tab as { type: "JsEditor"; codeIdx: number };
                  projectDispatch({
                    type: "UpdateJsFile",
                    codeIdx: t.codeIdx,
                    content: value,
                  });
                }}
              ></CodeMirror>
            </div>
            <div className="border-t-1 border-l-1 ring-inset-0 mr-0 h-full w-1/5 rounded bg-gray-50 shadow-sm ring-1 ring-gray-300">
              <HttpEndpoints></HttpEndpoints>
            </div>
          </div>
        );
      }
      case "Database": {
        return <Database></Database>;
      }
    }
  };

  return projectState.curOpenTabIdx === null ? (
    <></>
  ) : (
    renderTabContent(projectState.curOpenTabIdx)
  );
}
