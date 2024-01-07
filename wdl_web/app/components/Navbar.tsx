import Link from "next/link"
import { JSX, SVGProps } from "react"

export default function Navbar() {
    return (
        <header className="flex h-16 items-center px-4 md:px-6 border-b">
        <Link className="mr-6" href="#">
        <MountainIcon className="h-6 w-6" />
        <span className="sr-only">Acme Inc</span>
        </Link>
        <nav className="flex gap-4 sm:gap-6">
        <Link className="text-sm font-medium hover:underline underline-offset-4" href="#">
            Dashboard
        </Link>
        <Link className="text-sm font-medium hover:underline underline-offset-4" href="#">
            Reports
        </Link>
        <Link className="text-sm font-medium hover:underline underline-offset-4" href="#">
            Settings
        </Link>
        <Link className="text-sm font-medium hover:underline underline-offset-4" href="#">
            Contact
        </Link>
        </nav>
    </header>
    )
}

function MountainIcon(props: JSX.IntrinsicAttributes & SVGProps<SVGSVGElement>) {
    return (
      <svg
        {...props}
        xmlns="http://www.w3.org/2000/svg"
        width="24"
        height="24"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="m8 3 4 8 5-5 5 15H2L8 3z" />
      </svg>
    )
  }