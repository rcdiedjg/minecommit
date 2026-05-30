import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import { Button } from "@/components/ui/button"

export function RollingLogDialog() {
  return (
    <AlertDialog open={true}>
      <AlertDialogTrigger
        render={<Button variant="outline">Show Dialog</Button>}
      />
      <AlertDialogContent className="fixed min-h-4/5 min-w-4/5 grid-rows-[auto_1fr_auto] flex-row">
        <AlertDialogHeader>
          <AlertDialogTitle>运行日志</AlertDialogTitle>
          <AlertDialogDescription>请耐心等待运行结束</AlertDialogDescription>
        </AlertDialogHeader>
        <div className="bg-secondary"></div>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={true}>关闭</AlertDialogCancel>
          <AlertDialogAction variant="destructive">强制停止</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
