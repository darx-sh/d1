import React, { Fragment, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import {
  HttpRoute,
  useProjectDispatch,
  useProjectState,
} from "~/components/project_v2/ProjectContext";
import { githubDark, githubLight } from "@uiw/codemirror-theme-github";
import { json } from "@codemirror/lang-json";
import { EditorView } from "@codemirror/view";
import CodeMirror from "@uiw/react-codemirror";
import { PlayCircleIcon, PlayIcon } from "@heroicons/react/24/solid";
import { XMarkIcon, ClipboardIcon } from "@heroicons/react/24/outline";
import axios from "axios";
import { env } from "~/env.mjs";

type InvokeModalProps = {
  httpRoute: HttpRoute;
  onClose: () => void;
};

const myTheme = EditorView.theme({
  "&": {
    fontSize: "1rem",
    lineHeight: "1.5rem",
  },
});

export default function InvokeModal(props: InvokeModalProps) {
  console.log(`invoke modal, httpRoute = ${props.httpRoute.curParams}`);
  const [open, setOpen] = useState(true);
  const projectState = useProjectState()!;
  const projectDispatch = useProjectDispatch()!;
  const [postResult, setPostResult] = useState<string | null>(null);

  const functionUrl = `${env.NEXT_PUBLIC_DATA_PLANE_URL}/invoke/${props.httpRoute.httpPath}`;
  const url = new URL(functionUrl);
  // todo: hardcode env id for now.
  const envId = "cljb3ovlt0002e38vwo0xi5ge";
  let curlCommand = "";
  if (url.hostname === "localhost" || url.hostname === "127.0.0.1") {
    curlCommand = `curl -i -X POST -H "Content-Type: application/json" -H "Darx-Dev-Host: ${envId}.darx.sh" -d '${props.httpRoute.curParams}' ${functionUrl}`;
  } else {
    curlCommand = `curl -i -X POST -H "Content-Type: application/json" -d '${props.httpRoute.curParams}' ${functionUrl}`;
  }

  const handleInvoke = () => {
    setPostResult(null);
    // use projectState to fetch newest data.
    const r = projectState.directory.httpRoutes.filter((r) => {
      return r.httpPath === props.httpRoute.httpPath;
    })[0]!;
    axios
      .post(functionUrl, JSON.parse(r.curParams), {
        headers: { "Darx-Dev-Host": `${envId}.darx.sh` },
      })
      .then((response) => {
        setPostResult(JSON.stringify(response.data));
      })
      .catch((error) => console.log("invoke function error: ", error));
  };

  const myTheme = EditorView.theme({
    "&": {
      fontSize: "1rem",
      lineHeight: "1.5rem",
      maxHeight: "670px",
    },
    "&.cm-focused": {
      outline: "none",
    },
    ".cm-scroller": { overflow: "auto" },
  });

  return (
    <Transition.Root show={open} as={Fragment}>
      <Dialog
        as="div"
        className="relative z-10"
        onClose={() => {
          setOpen(false);
          props.onClose();
        }}
      >
        <div className="fixed inset-0" />
        <div className="fixed inset-0 overflow-hidden">
          <div className="absolute inset-0 overflow-hidden">
            <div className="pointer-events-none fixed inset-y-0 right-0 flex max-w-full pl-10">
              <Transition.Child
                as={Fragment}
                enter="transform transition ease-in-out duration-500 sm:duration-700"
                enterFrom="translate-x-full"
                enterTo="translate-x-0"
                leave="transform transition ease-in-out duration-500 sm:duration-700"
                leaveFrom="translate-x-0"
                leaveTo="translate-x-full"
              >
                <Dialog.Panel className="pointer-events-auto w-screen max-w-md">
                  <div className="flex h-full flex-col overflow-y-scroll border-l-2 bg-white py-3 shadow-xl">
                    <div className="px-4 sm:px-6">
                      <div className="flex items-start justify-between">
                        <Dialog.Title className="flex text-base text-gray-900">
                          <div className="text-small p-2 font-light">POST</div>
                          <div className="text-small p-2 font-normal">
                            {"/" + props.httpRoute.httpPath}
                          </div>
                        </Dialog.Title>
                        <div className="ml-3 flex h-7 items-center">
                          <button
                            type="button"
                            className="rounded-md bg-white text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
                            onClick={() => {
                              setOpen(false);
                              props.onClose();
                            }}
                          >
                            <span className="sr-only">Close panel</span>
                            <XMarkIcon className="h-6 w-6" aria-hidden="true" />
                          </button>
                        </div>
                      </div>
                    </div>
                    <div className="relative mt-3 flex-1 px-4 sm:px-6">
                      <div className=" border-2 p-2 shadow-md">
                        <div className="p-2 text-xs font-light">JSON Body</div>
                        <CodeMirror
                          value={props.httpRoute.curParams}
                          theme={githubLight}
                          extensions={[
                            json(),
                            myTheme,
                            EditorView.lineWrapping,
                          ]}
                          height="200px"
                          basicSetup={{ lineNumbers: false, foldGutter: false }}
                          onChange={(value, viewUpdate) => {
                            projectDispatch({
                              type: "UpdatePostParam",
                              httpRoute: props.httpRoute,
                              param: value,
                            });
                          }}
                        ></CodeMirror>
                        <button
                          type="button"
                          className="mt-2 rounded-md bg-indigo-600 px-2.5 py-1.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
                          onClick={handleInvoke}
                        >
                          Send Request
                        </button>
                      </div>
                      <div className="mt-3 max-w-fit overflow-x-auto">
                        <div className="flex items-center justify-end">
                          <p className="p-1 text-sm font-light italic">
                            Example curl command
                          </p>
                        </div>
                        <p className="whitespace-nowrap bg-gray-100 p-4 text-sm font-light italic">
                          {curlCommand}
                        </p>
                      </div>
                    </div>
                    {postResult && (
                      <div className="relative mt-6 flex-1 px-4 sm:px-6">
                        <div className="p-2 font-light">Result</div>
                        <CodeMirror
                          value={JSON.stringify(
                            JSON.parse(postResult),
                            null,
                            2
                          )}
                          theme={githubDark}
                          extensions={[
                            json(),
                            myTheme,
                            EditorView.lineWrapping,
                          ]}
                          height="200px"
                          readOnly={true}
                          basicSetup={{
                            lineNumbers: false,
                            foldGutter: false,
                            highlightActiveLine: false,
                          }}
                        ></CodeMirror>
                      </div>
                    )}
                  </div>
                </Dialog.Panel>
              </Transition.Child>
            </div>
          </div>
        </div>
      </Dialog>
    </Transition.Root>
  );
}
