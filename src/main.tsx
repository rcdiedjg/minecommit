import { StrictMode } from "react"
import { createRoot } from "react-dom/client"

import "./index.css"
import App from "./App.tsx"
import { ThemeProvider } from "@/components/theme-provider.tsx"
import { TooltipProvider } from "@/components/ui/tooltip.tsx"
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar.tsx"
import { AppSidebar } from "@/components/app-sidebar.tsx"

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider>
      <TooltipProvider>
        <SidebarProvider>
          <AppSidebar />
          <main>
            <SidebarTrigger />
            <App />
          </main>
        </SidebarProvider>
      </TooltipProvider>
    </ThemeProvider>
  </StrictMode>
)
