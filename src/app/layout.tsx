import type { Metadata } from "next";
import { Fraunces, Newsreader } from "next/font/google";
import "./globals.css";

const bodyFont = Newsreader({
  variable: "--font-body-face",
  subsets: ["latin"],
  weight: ["300", "400", "500", "600"],
});

const displayFont = Fraunces({
  variable: "--font-display-face",
  subsets: ["latin"],
  weight: ["400", "600", "700"],
});

export const metadata: Metadata = {
  title: "Pomodoro Bar",
  description: "状态栏番茄钟 · 专注、休息与节律",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="zh-CN">
      <body
        className={`${bodyFont.variable} ${displayFont.variable} antialiased`}
      >
        {children}
      </body>
    </html>
  );
}
