export const extractTags = (content: string) => (content.match(/#[\w-]+/g) ?? []).map((t) => t.slice(1));
