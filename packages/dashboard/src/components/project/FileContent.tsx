import JsEditor from "./JsEditor";

type FileContentProps = {
  name: string;
  content: string;
  handleCodeChange: (code: string) => void;
};
export default function FileContent(props: FileContentProps) {
  return (
    <div className="divide-y divide-gray-300">
      <div>tab: {props.name}</div>
      <JsEditor
        initialCode={props.content}
        readOnly={false}
        handleCodeChange={props.handleCodeChange}
      ></JsEditor>
    </div>
  );
}
