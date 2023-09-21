import {
  useProjectState,
  useProjectDispatch,
  CodeInfo,
} from "~/components/project/ProjectContext";
import Editor from "@monaco-editor/react";
import { loader } from "@monaco-editor/react";
import React from "react";
import HttpEndpoints from "~/components/project/functions/HttpEndpoints";
import LeftDirectory from "~/components/project/functions/LeftDirectory";
import EditorTabs from "~/components/project/functions/EditorTabs";
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

export default function Functions() {
  const state = useProjectState();
  const curOpenTab = state.curOpenTabIdx;

  function findCodeIdx(tab: number | null) {
    if (tab === null) {
      return null;
    }
    return state.tabs[tab]?.codeIdx;
  }

  const codeIdx = findCodeIdx(curOpenTab);
  let code: CodeInfo | null = null;
  if (codeIdx !== null && codeIdx !== undefined) {
    code = state.directory.codes[codeIdx]!;
  }

  return (
    <>
      <div className="relative h-full">
        <div className="absolute bottom-0 left-0 top-0 w-40 border-r bg-gray-50">
          <div className="flex flex-col">
            <LeftDirectory></LeftDirectory>
            <div className="relative mt-40">
              <div
                className="absolute inset-0 flex items-center"
                aria-hidden="true"
              >
                <div className="mx-2 w-full border-t border-gray-300" />
              </div>
            </div>
            <div
              className={classNames(
                true
                  ? "bg-gray-200 text-indigo-600"
                  : "text-gray-700 hover:bg-gray-50 hover:text-indigo-600",
                "mx-2 mt-4 flex items-center justify-center border p-2 py-3"
              )}
            >
              <ListBulletIcon className="h-5 w-5" />
              <div className="px-2 text-sm">API List</div>
            </div>
          </div>
        </div>
        <div className="pl-40">
          <EditorTabs></EditorTabs>
        </div>
        <div className="h-full pl-40 pr-64 pt-2">
          {code && <MyEditor code={code} codeIdx={codeIdx!}></MyEditor>}
        </div>
        <div className="absolute bottom-0 right-0 mr-0 h-full w-64 border-l bg-gray-50">
          <HttpEndpoints></HttpEndpoints>
        </div>
      </div>
    </>
  );
}
