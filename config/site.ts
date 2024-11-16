export type SiteConfig = typeof siteConfig;

export const siteConfig = {
  name: "Hectic",
  description: "Manage your expenses and more",
  navItems: [
    {
      label: "Home",
      href: "/",
    },
    {
      label: "Expenses",
      href: "/expenses",
    },
    {
      label: "Summary",
      href: "/summary",
    },
    {
      label: "Configure",
      href: "/configure",
    },
  ],
  navMenuItems: [
    {
      label: "Profile",
      href: "/profile",
    },
    {
      label: "Dashboard",
      href: "/dashboard",
    },
    {
      label: "Projects",
      href: "/projects",
    },
    {
      label: "Team",
      href: "/team",
    },
    {
      label: "Calendar",
      href: "/calendar",
    },
    {
      label: "Settings",
      href: "/settings",
    },
    {
      label: "Help & Feedback",
      href: "/help-feedback",
    },
    {
      label: "Logout",
      href: "/logout",
    },
  ],
  links: {
    github: "https://github.com/nextui-org/nextui",
    twitter: "https://twitter.com/getnextui",
    docs: "https://nextui.org",
    discord: "https://discord.gg/9b6yyZKmH4",
    sponsor: "https://patreon.com/jrgarciadev",
  },
};
