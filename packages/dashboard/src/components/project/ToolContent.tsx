import React from "react";
import { useProjectState, useProjectDispatch } from "./ProjectContext";
import HttpEndpoints from "~/components/project/HttpEndpoints";
import Database from "~/components/project/database/Database";
import Editor from "@monaco-editor/react";

export default function ToolContent() {
  const projectState = useProjectState()!;
  const projectDispatch = useProjectDispatch()!;

  const renderTabContent = (tabIdx: number) => {
    const tab = projectState.tabs[tabIdx]!;
    switch (tab.type) {
      case "JsEditor": {
        const code = projectState.directory.codes[tab.codeIdx]!;
        return (
          <div className="flex h-full justify-end space-x-2 bg-white">
            <div className="flex-1">
              <Editor
                defaultLanguage="javascript"
                value={code.content}
                path={code.fsPath}
                options={{
                  fontSize: 14,
                  minimap: { enabled: false },
                  overviewRulerBorder: false,
                }}
                onChange={(value, event) => {
                  const t = tab as { type: "JsEditor"; codeIdx: number };
                  projectDispatch({
                    type: "UpdateJsFile",
                    codeIdx: t.codeIdx,
                    content: value!,
                  });
                }}
              />
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
