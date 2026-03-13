// src/routes/document/[id]/+page.ts

export const prerender = false;

export const load = ({ params }: { params: { id: string } }) => {
	return {
		documentId: Number(params.id)
	};
};
