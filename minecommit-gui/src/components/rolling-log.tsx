import { useEffect, useMemo, useRef } from "react"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"

export type Operation = "commit" | "restore" | "push" | "pull"

function RollingLogContent({
  externalLines,
  externalFinished,
  onForceStop,
}: {
  externalLines?: string[]
  externalFinished?: boolean
  onForceStop?: () => void
}) {
  const preRef = useRef<HTMLPreElement>(null)
  const finished = externalFinished ?? false
  const lines = useMemo(() => externalLines ?? [], [externalLines])

  // Auto-scroll to bottom when lines change
  useEffect(() => {
    if (preRef.current) {
      preRef.current.scrollTop = preRef.current.scrollHeight
    }
  }, [lines])

  return (
    <>
      <AlertDialogHeader>
        <AlertDialogTitle>运行日志</AlertDialogTitle>
        <AlertDialogDescription>
          {finished ? "运行结束" : "请耐心等待运行结束..."}
        </AlertDialogDescription>
      </AlertDialogHeader>
      <pre
        ref={preRef}
        className="min-h-0 overflow-y-auto rounded-md bg-secondary p-4 font-mono text-sm whitespace-pre-wrap text-secondary-foreground"
      >
        {lines.join("\n")}
      </pre>
      <AlertDialogFooter>
        <AlertDialogCancel disabled={!finished}>关闭</AlertDialogCancel>
        <AlertDialogAction variant="destructive" onClick={onForceStop}>
          强制停止
        </AlertDialogAction>
      </AlertDialogFooter>
    </>
  )
}

export function RollingLogDialog({
  open,
  onOpenChange,
  operation,
  logs,
  finished,
  onForceStop,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  operation: Operation
  logs?: string[]
  finished?: boolean
  onForceStop?: () => void
}) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent className="fixed h-4/5 min-w-4/5 grid-rows-[auto_1fr_auto] flex-col">
        {open && (
          <RollingLogContent
            key={operation}
            externalLines={logs}
            externalFinished={finished}
            onForceStop={onForceStop}
          />
        )}
      </AlertDialogContent>
    </AlertDialog>
  )
}
