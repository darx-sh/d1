import CodeMirror from "@uiw/react-codemirror";
import { githubLight } from "@uiw/codemirror-theme-github";
import { javascript } from "@codemirror/lang-javascript";
import { EditorView } from "@codemirror/view";
import { useProjectState, useProjectDispatch } from "./ProjectContext";
import HttpEndpoints from "~/components/project_v2/HttpEndpoints";
import React from "react";

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
  const projectState = useProjectState();
  const projectDispatch = useProjectDispatch();

  const renderTabContent = (tabIdx: number) => {
    const tab = projectState!.tabs[tabIdx]!;
    switch (tab.type) {
      case "JsEditor": {
        const code = projectState!.directory.codes[tab.codeIdx];
        return (
          <div className="flex h-full justify-end space-x-2">
            <div className="flex-1">
              <CodeMirror
                value={code!.content}
                theme={githubLight}
                extensions={[javascript(), myTheme, EditorView.lineWrapping]}
                onChange={(value, viewUpdate) => {
                  const t = tab as { type: "JsEditor"; codeIdx: number };
                  projectDispatch!({
                    type: "UpdateJsFile",
                    codeIdx: t.codeIdx,
                    content: value,
                  });
                }}
              ></CodeMirror>
            </div>
            <div className="mr-0 h-full w-1/5 border-l-2 border-t-2 border-gray-300 bg-gray-50">
              <HttpEndpoints></HttpEndpoints>
            </div>
          </div>
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
