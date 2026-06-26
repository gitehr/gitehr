import { createTheme, type MantineColorsTuple } from "@mantine/core";

const clinical: MantineColorsTuple = [
  "#e8f4f8",
  "#d0e8f0",
  "#a0d0e0",
  "#6eb8d0",
  "#45a3c0",
  "#2b93b2",
  "#1d7f9e",
  "#156a85",
  "#0e566c",
  "#084255",
];

export const theme = createTheme({
  primaryColor: "clinical",
  colors: { clinical },
  fontFamily: "Inter, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif",
  fontFamilyMonospace: "JetBrains Mono, ui-monospace, SFMono-Regular, Menlo, monospace",
  defaultRadius: "sm",
  headings: {
    fontFamily: "Inter, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif",
    fontWeight: "600",
  },
});
