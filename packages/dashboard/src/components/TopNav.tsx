import Link from "next/link";

export interface TopNaveProps {
  navs: { name: string; href: string }[];
}

export default function TopNav(props: TopNaveProps) {
  const { navs } = props;

  return (
    <nav
      className="ml-2 flex rounded border-b bg-white"
      aria-label="Breadcrumb"
    >
      <ol role="list" className="flex items-center space-x-4">
        {navs.map((page, index) => (
          <li key={page.name}>
            <div className="flex h-12 items-center ">
              {index != 0 && (
                <svg
                  className="h-5 w-5 flex-shrink-0 text-gray-300"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                  aria-hidden="true"
                >
                  <path d="M5.555 17.776l8-16 .894.448-8 16-.894-.448z" />
                </svg>
              )}
              <Link
                href={page.href}
                className="ml-4 text-sm font-normal text-gray-500 hover:text-gray-900"
              >
                {page.name}
              </Link>
            </div>
          </li>
        ))}
      </ol>
    </nav>
  );
}
