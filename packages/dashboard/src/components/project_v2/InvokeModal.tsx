import React, { Fragment, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { HttpRoute } from "~/components/project_v2/ProjectContext";
import { githubLight } from "@uiw/codemirror-theme-github";
import { json } from "@codemirror/lang-json";
import { EditorView } from "@codemirror/view";
import CodeMirror from "@uiw/react-codemirror";
import { PlayCircleIcon } from "@heroicons/react/24/solid";
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
  const [open, setOpen] = useState(true);
  const [postParams, setPostParams] = useState<string>("{\n\n}");
  const [postResult, setPostResult] = useState<string | null>(null);

  const functionUrl = `${env.NEXT_PUBLIC_DATA_PLANE_URL}/invoke/${props.httpRoute.httpPath}`;
  const url = new URL(functionUrl);
  // todo: hardcode env id for now.
  const envId = "cljb3ovlt0002e38vwo0xi5ge";
  let curlCommand = "";
  if (url.hostname === "localhost" || url.hostname === "127.0.0.1") {
    curlCommand = `curl -i -X POST -H "Content-Type: application/json" -H "Darx-Dev-Host: ${envId}.darx.sh" -d '${postParams}' ${functionUrl}`;
  } else {
    curlCommand = `curl -i -X POST -H "Content-Type: application/json" -d '${postParams}' ${functionUrl}`;
  }

  const handleInvoke = () => {
    axios
      .post(functionUrl, JSON.parse(postParams), {
        headers: { "Darx-Dev-Host": `${envId}.darx.sh` },
      })
      .then((response) => {
        console.log("invoke function response: ", response.data);
        setPostResult(JSON.stringify(response.data));
      })
      .catch((error) => console.log("invoke function error: ", error));
  };

  return (
    <Transition.Root show={open} as={Fragment}>
      <Dialog as="div" className="relative z-10" onClose={props.onClose}>
        <Transition.Child
          as={Fragment}
          enter="ease-out duration-300"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" />
        </Transition.Child>

        <div className="fixed inset-0 z-10">
          <div className="flex h-full items-end items-center justify-center p-0 px-4 py-9 text-center">
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-300"
              enterFrom="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
              enterTo="opacity-100 translate-y-0 sm:scale-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100 translate-y-0 sm:scale-100"
              leaveTo="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
            >
              <Dialog.Panel className="relative h-full w-10/12 transform overflow-hidden rounded-lg bg-white px-4 pb-5 pt-5 text-left shadow-xl transition-all">
                <form>
                  <div className="space-y-12">
                    <div className="flex border-b border-gray-900/10">
                      <h2 className="px-5 text-base font-semibold  text-gray-900">
                        File Name: {"/" + props.httpRoute.jsEntryPoint}
                      </h2>
                      <h2 className="px-5 text-base font-semibold text-gray-900">
                        Export Name: {props.httpRoute.jsExport}
                      </h2>
                    </div>
                    <div className="border-b border-gray-900/10 px-5">
                      Http endpoint: {"/" + props.httpRoute.httpPath}
                    </div>
                    <div className="mt-2">
                      <div className="flex items-center">
                        <div className="p-2">Parameters</div>
                      </div>
                      <CodeMirror
                        value={postParams}
                        theme={githubLight}
                        extensions={[json(), myTheme]}
                        onChange={(value, viewUpdate) => {
                          setPostParams(value);
                        }}
                      ></CodeMirror>
                      <div className="flex pt-2">
                        <p>Invoke http</p>
                        <PlayCircleIcon
                          className="h-8 w-8 fill-blue-500"
                          onClick={() => handleInvoke()}
                        ></PlayCircleIcon>
                      </div>
                      <div>{curlCommand}</div>
                      <div>{postResult}</div>
                    </div>
                  </div>
                </form>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </Dialog>
    </Transition.Root>
  );
}
