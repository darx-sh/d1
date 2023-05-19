import React from "react";
import CodeMirror from "@uiw/react-codemirror";
import { javascript } from "@codemirror/lang-javascript";
import { ViewUpdate } from "@codemirror/view";

type JsEditorProps = {
  initialCode: string;
  readOnly: boolean;
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
      readOnly={props.readOnly}
      height="300px"
      width="800px"
      extensions={[javascript()]}
      onChange={onChange}
    />
  );
}
export default JsEditor;
