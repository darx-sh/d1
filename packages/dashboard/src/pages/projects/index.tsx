import Link from "next/link";
import { useRouter } from "next/router";
import { useState } from "react";
import { useEffectOnce } from "usehooks-ts";
import axios from "axios";
import axiosRetry from "axios-retry";
import Spinner from "~/components/project/Spinner";
import {
  ProjectInfo,
  ProjectProvider,
  EnvInfo,
} from "~/components/project/ProjectContext";
import randomName from "~/components/project/RandomName";

const user = {
  id: "Tom Cook",
  email: "tom@example.com",
  imageUrl:
    "https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80",
};
const navigation = [{ id: "Projects", href: "#", current: true }];
const userNavigation = [{ id: "Sign out", href: "#" }];

const orgId = "org_123";

function classNames(...classes: any[]) {
  return classes.filter(Boolean).join(" ");
}

type ListProjectRsp = {
  projects: ProjectInfo[];
};

type NewProjectRsp = {
  project: ProjectInfo;
  env: EnvInfo;
};

function Projects() {
  const [isLoading, setIsLoading] = useState(true);
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const router = useRouter();

  const navToProject = (project: ProjectInfo) => {
    // eslint-disable-next-line
    router.push(`/projects/${project.id}`);
  };

  const handleNewProject = () => {
    const url = "http://localhost:3457/new_tenant_project";
    axios
      .post(url, { org_id: orgId, project_name: randomName() })
      .then((response) => {
        const { project } = response.data as NewProjectRsp;
        navToProject(project);
      })
      .catch((err) => {
        console.error("failed to create project");
      });
  };

  useEffectOnce(() => {
    const listProjectUrl = `http://localhost:3457/list_project/${orgId}`;
    const instance = axios.create();
    axiosRetry(instance, {
      retries: 100,
      shouldResetTimeout: true,
      retryDelay: (retryCount) => {
        return 1000;
      },
    });
    instance
      .get(listProjectUrl, { timeout: 4000 })
      .then((response) => {
        const { projects } = response.data as ListProjectRsp;
        setIsLoading(false);
        setProjects(projects);
      })
      .catch((err) => {
        console.error(err);
      });
  });

  return (
    <>
      {isLoading ? (
        <Spinner />
      ) : (
        <div className="min-h-full">
          <nav className="border-b border-gray-200 bg-white">
            <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
              <div className="flex h-16 justify-between">
                <div className="flex">
                  <div className="flex flex-shrink-0 items-center">
                    <img
                      className="h-8 w-auto"
                      src="https://tailwindui.com/img/logos/mark.svg?color=indigo&shade=600"
                      alt="Darx Logo"
                    />
                  </div>
                  <div className="sm:-my-px sm:ml-6 sm:flex sm:space-x-8">
                    {navigation.map((item) => (
                      <a
                        key={item.id}
                        href={item.href}
                        className={classNames(
                          item.current
                            ? "border-indigo-500 text-gray-900"
                            : "border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700",
                          "inline-flex items-center border-b-2 px-1 pt-1 text-sm font-medium"
                        )}
                        aria-current={item.current ? "page" : undefined}
                      >
                        {item.id}
                      </a>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          </nav>

          <div className="my-10 px-60">
            <button
              onClick={handleNewProject}
              type="button"
              className="mb-12 rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
            >
              New Project
            </button>
            <ul role="list" className="grid grid-cols-2 gap-4">
              {projects.map((project) => (
                <li
                  key={project.id}
                  className="rounded-lg border border-gray-300 transition-colors duration-200 hover:border-gray-700"
                >
                  <Link
                    onClick={(e) => {
                      e.preventDefault();
                      navToProject(project);
                    }}
                    href={`/projects/${project.id}}`}
                    className="flex place-content-center py-5"
                  >
                    <h2>
                      <strong className="align-middle text-base font-medium leading-tight">
                        {project.name}
                      </strong>
                    </h2>
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </>
  );
}

export default function ProjectsWrapper() {
  return (
    <ProjectProvider>
      <Projects></Projects>
    </ProjectProvider>
  );
}
