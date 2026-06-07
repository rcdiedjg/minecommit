import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/ui/card"
import {
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
  TableCell,
} from "@/components/ui/table"
import {
  HoverCard,
  HoverCardTrigger,
  HoverCardContent,
} from "@/components/ui/hover-card"
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { useState, useEffect, useCallback } from "react"
import { Trash2, HardDrive, FolderOpen } from "lucide-react"
import { invoke } from "@tauri-apps/api/core"
import { open as openFolderDialog } from "@tauri-apps/plugin-dialog"

interface Save {
  name: string
  path: string
  repo_path: string
  remote_repo_path: string
}

function EmptySave({ onAddTrack }: { onAddTrack: () => void }) {
  return (
    <Empty>
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <HardDrive />
        </EmptyMedia>
        <EmptyTitle>跟踪一个存档</EmptyTitle>
        <EmptyDescription>
          <p>MineCommit 还没有跟踪任何存档</p>
          <p>点击按钮来跟踪一个已有的存档</p>
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <Button onClick={onAddTrack}>添加跟踪</Button>
      </EmptyContent>
    </Empty>
  )
}

/** Derive a save name from a folder path (e.g. last segment). */
function deriveNameFromPath(path: string): string {
  const normalized = path.replace(/\\/g, "/")
  const parts = normalized.split("/")
  return parts[parts.length - 1] || ""
}

/** Derive a reasonable default local repo path from the save path. */
function deriveRepoPathFromPath(path: string): string {
  const name = deriveNameFromPath(path)
  const normalized = path.replace(/\\/g, "/")
  const parentDir = normalized.split("/").slice(0, -1).join("/")
  return `${parentDir}/minecommit/${name}.git`
}

type AddTrackStep = "select" | "confirm"

function AddTrackDialog({
  open,
  onOpenChange,
  onSaveAdded,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSaveAdded: () => void
}) {
  const [step, setStep] = useState<AddTrackStep>("select")

  // Form state (pre-filled after folder selection)
  const [name, setName] = useState("")
  const [path, setPath] = useState("")
  const [localRepoPath, setLocalRepoPath] = useState("")
  const [remoteRepoPath, setRemoteRepoPath] = useState("")
  const [error, setError] = useState("")
  const [submitting, setSubmitting] = useState(false)
  const [selecting, setSelecting] = useState(false)

  function resetAll() {
    setStep("select")
    setName("")
    setPath("")
    setLocalRepoPath("")
    setRemoteRepoPath("")
    setError("")
    setSubmitting(false)
    setSelecting(false)
  }

  function handleOpenChange(open: boolean) {
    if (!open) {
      resetAll()
    }
    onOpenChange(open)
  }

  // --- Step: select ---
  async function handleSelectFolder() {
    setSelecting(true)
    try {
      const selected = await openFolderDialog({
        directory: true,
        multiple: false,
        title: "选择存档文件夹",
      })
      if (selected) {
        // Pre-fill fields
        setName(deriveNameFromPath(selected))
        setPath(selected)
        setLocalRepoPath(deriveRepoPathFromPath(selected))
        setRemoteRepoPath("")
        setError("")
        setStep("confirm")
      }
    } catch (err) {
      console.error("Failed to open folder dialog:", err)
    } finally {
      setSelecting(false)
    }
  }

  // --- Step: confirm ---
  async function handleSubmit(e: { preventDefault: () => void }) {
    e.preventDefault()
    setError("")
    setSubmitting(true)
    try {
      await invoke("add_save", {
        name,
        path,
        repoPath: localRepoPath,
        remoteRepoPath,
      })
      onOpenChange(false)
      resetAll()
      onSaveAdded()
    } catch (err) {
      setError(String(err))
    } finally {
      setSubmitting(false)
    }
  }

  function handleBack() {
    resetAll()
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        {step === "select" && (
          <>
            <DialogHeader>
              <DialogTitle>选择存档</DialogTitle>
              <DialogDescription>
                选择一个存档文件夹，需包含 level.dat 文件
              </DialogDescription>
            </DialogHeader>
            <div className="flex flex-col items-center gap-4 py-4">
              <Button
                size="lg"
                disabled={selecting}
                onClick={handleSelectFolder}
                className="w-full max-w-xs"
              >
                <FolderOpen data-icon="inline-start" />
                {selecting ? "请选择…" : "选择存档文件夹"}
              </Button>
            </div>
            <DialogFooter>
              <DialogClose render={<Button variant="outline" />}>
                取消
              </DialogClose>
            </DialogFooter>
          </>
        )}

        {step === "confirm" && (
          <form
            onSubmit={(e) => e.preventDefault()}
            className="flex flex-col gap-4"
          >
            <DialogHeader>
              <DialogTitle>确认存档信息</DialogTitle>
              <DialogDescription>
                已自动填写以下字段，请确认或修改后提交
              </DialogDescription>
            </DialogHeader>
            <FieldGroup>
              <Field>
                <FieldLabel htmlFor="save-name">存档名称</FieldLabel>
                <Input
                  id="save-name"
                  placeholder="我的世界"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  required
                />
              </Field>
              <Field>
                <FieldLabel htmlFor="save-path">存档路径</FieldLabel>
                <Input
                  id="save-path"
                  placeholder="/home/user/.minecraft/saves/我的世界"
                  value={path}
                  onChange={(e) => setPath(e.target.value)}
                  required
                />
              </Field>
              <Field>
                <FieldLabel htmlFor="local-repo-path">本地仓库路径</FieldLabel>
                <Input
                  id="local-repo-path"
                  placeholder="/home/user/.minecraft/minecommit/我的世界.git"
                  value={localRepoPath}
                  onChange={(e) => setLocalRepoPath(e.target.value)}
                />
              </Field>
              <Field>
                <FieldLabel htmlFor="remote-repo-path">
                  远程仓库路径（可选）
                </FieldLabel>
                <Input
                  id="remote-repo-path"
                  placeholder="https://git.example.com/我的世界.git"
                  value={remoteRepoPath}
                  onChange={(e) => setRemoteRepoPath(e.target.value)}
                />
              </Field>
              {error && <p className="text-sm text-destructive">{error}</p>}
            </FieldGroup>
            <DialogFooter className="mt-6">
              <Button variant="outline" type="button" onClick={handleBack}>
                返回
              </Button>
              <Button
                type="button"
                disabled={submitting}
                onClick={handleSubmit}
              >
                {submitting ? "添加中…" : "跟踪"}
              </Button>
            </DialogFooter>
          </form>
        )}
      </DialogContent>
    </Dialog>
  )
}

export function SaveManagePage() {
  const [saves, setSaves] = useState<Save[]>([])
  const [dialogOpen, setDialogOpen] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState("")

  const loadSaves = useCallback(async () => {
    try {
      setLoading(true)
      setError("")
      const data = await invoke<Save[]>("list_saves")
      setSaves(data)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    loadSaves()
  }, [loadSaves])

  async function handleDelete(name: string) {
    try {
      await invoke("delete_save", { name })
      await loadSaves()
    } catch (err) {
      setError(String(err))
    }
  }

  return (
    <div className="flex w-full flex-col gap-4 p-4">
      <Card>
        <CardHeader>
          <div className="flex items-end justify-between">
            <div>
              <CardTitle>存档列表</CardTitle>
              <CardDescription>管理 MineCommit 对存档的跟踪</CardDescription>
            </div>
            {saves.length > 0 && (
              <Button onClick={() => setDialogOpen(true)}>添加跟踪</Button>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {error && <p className="mb-4 text-sm text-destructive">{error}</p>}
          {loading ? (
            <p className="text-sm text-muted-foreground">加载中…</p>
          ) : saves.length === 0 ? (
            <EmptySave onAddTrack={() => setDialogOpen(true)} />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="text-muted-foreground">
                    存档名称
                  </TableHead>
                  <TableHead className="text-muted-foreground">
                    存档路径
                  </TableHead>
                  <TableHead>
                    <span className="sr-only">操作</span>
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {saves.map((save) => (
                  <HoverCard key={save.name}>
                    <HoverCardTrigger render={<TableRow />}>
                      <TableCell className="">{save.name}</TableCell>
                      <TableCell>{save.path}</TableCell>
                      <TableCell className="text-right">
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          className="cursor-pointer"
                          onClick={(e) => {
                            e.stopPropagation()
                            handleDelete(save.name)
                          }}
                        >
                          <Trash2 />
                        </Button>
                      </TableCell>
                    </HoverCardTrigger>
                    <HoverCardContent align="start" className="w-auto">
                      <div className="flex flex-col gap-3">
                        <div>
                          <p className="text-xs text-muted-foreground">
                            存档名称
                          </p>
                          <p className="font-medium">{save.name}</p>
                        </div>
                        <div>
                          <p className="text-xs text-muted-foreground">
                            存档路径
                          </p>
                          <p className="font-mono text-xs break-all">
                            {save.path}
                          </p>
                        </div>
                        <div>
                          <p className="text-xs text-muted-foreground">
                            仓库路径
                          </p>
                          <p className="font-mono text-xs break-all">
                            {save.repo_path}
                          </p>
                        </div>
                        <div>
                          <p className="text-xs text-muted-foreground">
                            远程仓库路径
                          </p>
                          {save.remote_repo_path ? (
                            <p className="font-mono text-xs break-all">
                              {save.remote_repo_path}
                            </p>
                          ) : (
                            <p className="font-mono text-xs break-all text-muted-foreground">
                              {"（未设置）"}
                            </p>
                          )}
                        </div>
                      </div>
                    </HoverCardContent>
                  </HoverCard>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <AddTrackDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onSaveAdded={loadSaves}
      />
    </div>
  )
}
