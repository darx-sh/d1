import CodeMirror from "@uiw/react-codemirror";
import { githubLight } from "@uiw/codemirror-theme-github";
import { javascript } from "@codemirror/lang-javascript";
import { EditorView } from "@codemirror/view";

const defaultCode = `\
export default function foo() {
  return "hello";
}`;

const myTheme = EditorView.theme({
  "&": {
    fontSize: "1rem",
    lineHeight: "1.5rem",
  },
});

export default function MiddleToolContent() {
  return (
    <CodeMirror
      value={defaultCode}
      theme={githubLight}
      extensions={[javascript(), myTheme]}
    ></CodeMirror>
  );
}
