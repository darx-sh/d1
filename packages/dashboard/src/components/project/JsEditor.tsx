import React from "react";
import CodeMirror from "@uiw/react-codemirror";
import { javascript } from "@codemirror/lang-javascript";
import { ViewUpdate } from "@codemirror/view";
import { githubLight } from "@uiw/codemirror-theme-github";

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
      minHeight="200px"
      width="768px"
      theme={githubLight}
      extensions={[javascript()]}
      onChange={onChange}
    />
  );
}
export default JsEditor;
