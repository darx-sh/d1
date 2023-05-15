import React from "react";
import CodeMirror from "@uiw/react-codemirror";
import { javascript } from "@codemirror/lang-javascript";
import { ViewUpdate } from "@codemirror/view";

type JsEditorProps = {
  initialCode: string;
};

function JsEditor(props: JsEditorProps) {
  const onChange = React.useCallback(
    (value: string, viewUpdate: ViewUpdate) => {
      console.log("value:", value);
    },
    []
  );

  return (
    <CodeMirror
      value={props.initialCode}
      height="200px"
      extensions={[javascript()]}
      onChange={onChange}
    />
  );
}
export default JsEditor;
