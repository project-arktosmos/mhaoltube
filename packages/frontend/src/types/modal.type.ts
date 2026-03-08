export type NavbarModalId =
	| 'youtube'
	| 'youtube-search'
	| 'libraries'
	| 'settings'
	| 'db'
	| 'yt-channels';

export interface ModalRouterState {
	navbarModal: NavbarModalId | null;
	mediaDetail: { type: string; category: string; id: string } | null;
}
