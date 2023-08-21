export default function LoadingBar() {
  return (
    <svg
      className="flex h-10 w-10 animate-spin items-center text-gray-500"
      viewBox="0 0 24 24"
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
      />
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm12 0a8 8 0 018 8V0c0-6.627-5.373-12-12-12h4zm-4 4a4 4 0 100 8 4 4 0 000-8zm0 6a2 2 0 110-4 2 2 0 010 4z"
      />
    </svg>
  );
}
