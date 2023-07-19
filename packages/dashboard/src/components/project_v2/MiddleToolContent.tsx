import CodeMirror from "@uiw/react-codemirror";
import { githubLight } from "@uiw/codemirror-theme-github";
import { javascript } from "@codemirror/lang-javascript";

const defaultCode = `\
export default function foo() {
  return "hello";
}`;

export default function MiddleToolContent() {
  return (
    <CodeMirror value={defaultCode} extensions={[javascript()]}></CodeMirror>
  );
}
