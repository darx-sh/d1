import {
  useProjectState,
  useProjectDispatch,
} from "~/components/project/ProjectContext";
import Editor from "@monaco-editor/react";
import React from "react";
import HttpEndpoints from "~/components/project/functions/HttpEndpoints";
import LeftDirectory from "~/components/project/functions/LeftDirectory";
import EditorTabs from "~/components/project/functions/EditorTabs";

function MyEditor() {
  const state = useProjectState();
  const dispatch = useProjectDispatch();
  const curOpenTab = state.curOpenTabIdx;

  function findCode(tab: number | null) {
    if (tab === null) {
      return null;
    } else {
      return state.directory.codes[tab];
    }
  }
  const code = findCode(curOpenTab);
  if (code === null || code === undefined) {
    return null;
  }

  return (
    <>
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
          dispatch({
            type: "UpdateJsFile",
            codeIdx: curOpenTab!,
            content: value!,
          });
        }}
      />
    </>
  );
}

export default function Functions() {
  const state = useProjectState();
  const curOpenTab = state.curOpenTabIdx;

  function findCode(tab: number | null) {
    if (tab === null) {
      return null;
    } else {
      return state.directory.codes[tab];
    }
  }
  const code = findCode(curOpenTab);
  return (
    <>
      <div className="relative h-full">
        <div className="absolute bottom-0 left-0 top-0 w-40 bg-gray-50">
          <LeftDirectory></LeftDirectory>
        </div>
        <div className="pl-40">
          <EditorTabs></EditorTabs>
        </div>
        <div className="h-full pl-40 pr-64">
          {code !== null && code !== undefined && <MyEditor></MyEditor>}
        </div>
        <div className="border-t-1 border-l-1 ring-inset-0 absolute bottom-0 right-0 mr-0 h-full w-64 rounded bg-gray-50 shadow-sm ring-1 ring-gray-300">
          <HttpEndpoints></HttpEndpoints>
        </div>
      </div>
    </>
  );
}
