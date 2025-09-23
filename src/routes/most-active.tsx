// import { commands, type UiUsageIntervalEntity } from '@/bindings'
// import { useQuery } from '@tanstack/react-query'
// import { createFileRoute } from '@tanstack/react-router'
// import { Badge } from '@/components/ui/badge'
// import { Button } from '@/components/ui/button'
// import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
// import { Skeleton } from '@/components/ui/skeleton'

// export const Route = createFileRoute('/most-active')({
//   component: TimelinePage,
// })

// function TimelinePage() {
//   const { data, isLoading, refetch, isFetching } = useQuery({
//     queryKey: ['timeline'],
//     queryFn: () => commands.getTimelineData(),
//   })

//   const result = data?.status === 'ok' ? data.data : undefined
//   const intervals = (result ?? [])
//     .map(([id, interval]) => ({ id, interval }))
//     .filter((x): x is { id: string; interval: UiUsageIntervalEntity } => Boolean(x.interval))
//     .sort((a, b) => new Date(b.interval.start).getTime() - new Date(a.interval.start).getTime())

//   console.log(data)
//   return (
//     <div className="mx-auto max-w-3xl p-6">
//       <div className="mb-6 flex items-center justify-between">
//         <div>
//           <h1 className="text-2xl font-bold">Usage Timeline</h1>
//           <p className="text-muted-foreground">What apps you used and when</p>
//         </div>
//         <Button onClick={() => refetch()} disabled={isFetching}>
//           {isFetching ? 'Refreshing…' : 'Refresh'}
//         </Button>
//       </div>

//       {isLoading ? (
//         <Card>
//           <CardHeader>
//             <CardTitle>
//               <Skeleton className="h-6 w-48" />
//             </CardTitle>
//             <CardDescription>
//               <Skeleton className="h-4 w-64" />
//             </CardDescription>
//           </CardHeader>
//           <CardContent className="space-y-3">
//             <Skeleton className="h-20 w-full" />
//             <Skeleton className="h-16 w-11/12" />
//             <Skeleton className="h-16 w-10/12" />
//           </CardContent>
//         </Card>
//       ) : intervals.length > 0 ? (
//         <Card className="shadow-sm">
//           <CardHeader>
//             <div className="flex items-center gap-2">
//               <Badge variant="secondary">Timeline</Badge>
//               <CardTitle className="leading-tight">Recent activity</CardTitle>
//             </div>
//             <CardDescription>
//               {intervals.length} intervals
//             </CardDescription>
//           </CardHeader>
//           <CardContent>
//             <div className="relative">
//               <div className="absolute left-3 top-0 h-full w-px bg-border" />
//               <ul className="space-y-4">
//                 {intervals.map(({ id, interval }) => (
//                   <li key={id} className="relative pl-10">
//                     <span className="absolute left-0 top-1.5 inline-block h-3 w-3 -translate-x-1/2 rounded-full border bg-background" />
//                     <div className="rounded-lg border p-3">
//                       <div className="flex items-center justify-between gap-2">
//                         <div className="flex min-w-0 flex-col">
//                           <div className="truncate font-medium">{interval.app_name ?? interval.app_identifier ?? 'Unknown App'}</div>
//                           <div className="truncate text-sm text-muted-foreground">{interval.window_name}</div>
//                         </div>
//                         <div className="shrink-0 text-xs text-muted-foreground">{formatTimeRange(interval.start, parseFloat(interval.duration))}</div>
//                       </div>
//                       <div className="mt-2 grid gap-2 text-xs text-muted-foreground sm:grid-cols-2">
//                         <Info label="Process" value={interval.process_path ?? '—'} />
//                         <Info label="Duration" value={formatDuration(parseFloat(interval.duration))} />
//                       </div>
//                     </div>
//                   </li>
//                 ))}
//               </ul>
//             </div>
//           </CardContent>
//         </Card>
//       ) : (
//         <EmptyState onRefresh={() => refetch()} />
//       )}
//     </div>
//   )
// }

// function Info(props: { label: string; value: string }) {
//   return (
//     <div className="rounded-md border bg-card p-3">
//       <div className="text-xs text-muted-foreground">{props.label}</div>
//       <div className="truncate text-sm font-medium">{props.value}</div>
//     </div>
//   )
// }

// function EmptyState(props: { onRefresh: () => void }) {
//   return (
//     <Card className="text-center">
//       <CardHeader>
//         <CardTitle>No activity yet</CardTitle>
//         <CardDescription>We couldn’t find any timeline data.</CardDescription>
//       </CardHeader>
//       <CardContent>
//         <Button onClick={props.onRefresh}>Try again</Button>
//       </CardContent>
//     </Card>
//   )
// }

// function formatDuration(totalSeconds: number) {
//   const hours = Math.floor(totalSeconds / 3600)
//   const minutes = Math.floor((totalSeconds % 3600) / 60)
//   const seconds = Math.floor(totalSeconds % 60)
//   const parts = [] as string[]
//   if (hours) parts.push(`${hours}h`)
//   if (minutes) parts.push(`${minutes}m`)
//   if (seconds || parts.length === 0) parts.push(`${seconds}s`)
//   return parts.join(' ')
// }

// function formatTimeRange(startIso: string, durationSeconds: number) {
//   const start = new Date(startIso)
//   const end = new Date(start.getTime() + durationSeconds * 1000)
//   const sameDay = start.toDateString() === end.toDateString()
//   const day = start.toLocaleDateString()
//   const startTime = start.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
//   const endTime = end.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
//   return sameDay ? `${day} · ${startTime}–${endTime}` : `${day} ${startTime} → ${end.toLocaleDateString()} ${endTime}`
// }

