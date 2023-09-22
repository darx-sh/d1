import {
  useProjectState,
  useProjectDispatch,
  CodeInfo,
} from "~/components/project/ProjectContext";
import Editor from "@monaco-editor/react";
import { loader } from "@monaco-editor/react";
import React from "react";
import HttpEndpoints from "~/components/project/playground/HttpEndpoints";
import LeftDirectory from "~/components/project/playground/LeftDirectory";
import EditorTabs from "~/components/project/playground/EditorTabs";
import classNames from "classnames";
import { ListBulletIcon } from "@heroicons/react/20/solid";

loader.config({
  paths: {
    vs: "https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.36.1/min/vs",
  },
});

interface MyEditorProps {
  code: CodeInfo;
  codeIdx: number;
}

function MyEditor(props: MyEditorProps) {
  const dispatch = useProjectDispatch();
  const { code, codeIdx } = props;

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
            codeIdx,
            content: value!,
          });
        }}
      />
    </>
  );
}

export default function Playground() {
  const state = useProjectState();
  const curOpenTab = state.curOpenTabIdx;

  function findCodeIdx(tabIdx: number | null) {
    if (tabIdx === null) {
      return null;
    }

    const tab = state.tabs[tabIdx];
    if (tab === undefined) {
      return null;
    }

    switch (tab.type) {
      case "JsEditor":
        return tab.codeIdx;
    }
  }

  const codeIdx = findCodeIdx(curOpenTab);
  let code: CodeInfo | null = null;
  if (codeIdx !== null && codeIdx !== undefined) {
    code = state.directory.codes[codeIdx]!;
  }

  return (
    <>
      <div className="relative h-full">
        <div className="absolute bottom-0 left-0 top-0 w-40 border-r">
          <div className="flex flex-col">
            <div className="px-2 py-2">
              <LeftDirectory></LeftDirectory>
            </div>
          </div>
        </div>
        <div className="pl-40">
          <EditorTabs></EditorTabs>
        </div>
        <div className="h-full pl-40 pr-64 pt-3.5">
          {code && <MyEditor code={code} codeIdx={codeIdx!}></MyEditor>}
        </div>
        <div className="absolute bottom-0 right-0 mr-0 h-full w-64 border-l bg-gray-50">
          <HttpEndpoints></HttpEndpoints>
        </div>
      </div>
    </>
  );
}
