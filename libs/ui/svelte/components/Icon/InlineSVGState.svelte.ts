export const svgs = $state<Record<string, string>>({});

export async function fetchSVG(src: string) {
  try {
    const response = await fetch(src);
    if (!response.ok) {
      throw new Error(`Failed to fetch SVG: ${response.statusText}`);
    }
    const svgText = await response.text();
    svgs[src] = svgText;
  } catch (e) {
    console.error(e);
  }
}
