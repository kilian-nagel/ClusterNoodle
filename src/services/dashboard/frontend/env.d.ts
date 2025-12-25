/// <reference types="vite/client" />

declare global {
	interface Window {
		__ENV?: {
			BACKEND_URL?: string
			// Support the common misspelling to be forgiving
			BACKEDN_URL?: string
		}
	}
}

export {}
