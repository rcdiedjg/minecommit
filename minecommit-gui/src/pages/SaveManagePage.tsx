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

interface Save {
  name: string
  path: string
}

const saves: Save[] = [
  { name: "世界1", path: "/home/user/.minecraft/saves/世界1" },
  { name: "创造测试", path: "/home/user/.minecraft/saves/创造测试" },
  { name: "红石实验室", path: "/home/user/.minecraft/saves/红石实验室" },
  { name: "生存存档", path: "/home/user/.minecraft/saves/生存存档" },
]

export function SaveManagePage() {
  return (
    <div className="flex w-full flex-col gap-4 p-4">
      <h1 className="text-2xl font-bold">存档管理</h1>

      <Card>
        <CardHeader>
          <CardTitle>存档列表</CardTitle>
          <CardDescription>管理你的 Minecraft 存档</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>存档名称</TableHead>
                <TableHead>存档路径</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {saves.map((save) => (
                <TableRow key={save.name}>
                  <TableCell className="font-medium">{save.name}</TableCell>
                  <TableCell className="font-mono text-xs text-muted-foreground">
                    {save.path}
                  </TableCell>
                </TableRow>
              ))}
              {saves.length === 0 && (
                <TableRow>
                  <TableCell
                    colSpan={2}
                    className="text-center text-muted-foreground"
                  >
                    暂无存档
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
