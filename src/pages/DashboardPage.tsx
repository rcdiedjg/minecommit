import { ActivityGraph } from "@/components/activity-graph"

const contributonData = [
  { date: "2026-03-11", count: 5 },
  { date: "2026-03-12", count: 5 },
  { date: "2026-03-13", count: 1 },
  { date: "2026-03-14", count: 5 },
  { date: "2026-03-15", count: 1 },
  { date: "2026-03-16", count: 0 },
  { date: "2026-03-17", count: 3 },
  { date: "2026-03-18", count: 5 },
  { date: "2026-03-19", count: 0 },
  { date: "2026-03-20", count: 2 },
  { date: "2026-03-21", count: 5 },
  { date: "2026-03-22", count: 3 },
  { date: "2026-03-23", count: 0 },
  { date: "2026-03-24", count: 3 },
  { date: "2026-03-25", count: 3 },
  { date: "2026-03-26", count: 5 },
  { date: "2026-03-27", count: 0 },
  { date: "2026-03-28", count: 5 },
  { date: "2026-03-29", count: 0 },
  { date: "2026-03-30", count: 5 },
]

export function DashboardPage() {
  return (
    <div className="flex w-full flex-row">
      <div className="flex w-full">
        <ActivityGraph data={contributonData} />
        <></>
      </div>
    </div>
  )
}
