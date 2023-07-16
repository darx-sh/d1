import React from "react";
import CodeMirror from "@uiw/react-codemirror";
import { javascript } from "@codemirror/lang-javascript";
import { ViewUpdate } from "@codemirror/view";

type JsEditorProps = {
  initialCode: string;
  handleCodeChange: (code: string) => void;
  readOnly: boolean;
};

function JsEditor(props: JsEditorProps) {
  const onChange = React.useCallback(
    (value: string, viewUpdate: ViewUpdate) => {
      props.handleCodeChange(value);
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
