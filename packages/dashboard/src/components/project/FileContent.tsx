import JsEditor from "./JsEditor";

type FileContentProps = {
  name: string;
  content: string;
};
export default function FileContent(props: FileContentProps) {
  return (
    <div className="divide-gray-300">
      <div>tab: {props.name}</div>
      <JsEditor initialCode={props.content} readOnly={false}></JsEditor>
    </div>
  );
}
