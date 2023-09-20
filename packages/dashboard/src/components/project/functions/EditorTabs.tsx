import {
  useProjectState,
  useProjectDispatch,
} from "~/components/project/ProjectContext";
import classNames from "classnames";

export default function EditorTabs() {
  const state = useProjectState();
  const dispatch = useProjectDispatch();
  const tabs = state.tabs.map((tab, idx) => {
    switch (tab.type) {
      case "JsEditor": {
        const codeIdx = tab.codeIdx;
        const fileName = state.directory.codes[codeIdx]!.fsPath;
        if (idx === state.curOpenTabIdx) {
          return {
            name: fileName,
            href: "#",
            current: true,
          };
        } else {
          return {
            name: fileName,
            href: "#",
            current: false,
          };
        }
      }
    }
  });

  const jsIcon = () => {
    return (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1em"
        viewBox="0 0 448 512"
      >
        <path d="M0 32v448h448V32H0zm243.8 349.4c0 43.6-25.6 63.5-62.9 63.5-33.7 0-53.2-17.4-63.2-38.5l34.3-20.7c6.6 11.7 12.6 21.6 27.1 21.6 13.8 0 22.6-5.4 22.6-26.5V237.7h42.1v143.7zm99.6 63.5c-39.1 0-64.4-18.6-76.7-43l34.3-19.8c9 14.7 20.8 25.6 41.5 25.6 17.4 0 28.6-8.7 28.6-20.8 0-14.4-11.4-19.5-30.7-28l-10.5-4.5c-30.4-12.9-50.5-29.2-50.5-63.5 0-31.6 24.1-55.6 61.6-55.6 26.8 0 46 9.3 59.8 33.7L368 290c-7.2-12.9-15-18-27.1-18-12.3 0-20.1 7.8-20.1 18 0 12.6 7.8 17.7 25.9 25.6l10.5 4.5c35.8 15.3 55.9 31 55.9 66.2 0 37.8-29.8 58.6-69.7 58.6z" />
      </svg>
    );
  };

  return (
    <nav className="flex border-b px-4" aria-label="Tabs">
      {tabs.map((tab, tabIdx) => (
        <a
          key={tab.name}
          href={tab.href}
          className={classNames(
            tab.current
              ? "border-x border-b-2 border-b-blue-300 text-gray-900"
              : " text-gray-500 hover:text-gray-700",
            "px-4 py-2 text-center text-xs focus:z-10"
          )}
          aria-current={tab.current ? "page" : undefined}
          onClick={() => {
            dispatch({
              type: "SelectTab",
              tabIdx,
            });
          }}
        >
          <div className="flex flex-row items-center space-x-1">
            {state.tabs[tabIdx]!.type === "JsEditor" ? jsIcon() : null}
            <p> {tab.name}</p>
          </div>
        </a>
      ))}
    </nav>
  );
}
