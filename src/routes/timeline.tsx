"use client"
import { createFileRoute } from '@tanstack/react-router'
import { useMemo, useState } from "react"
import { useQuery } from '@tanstack/react-query'
import { BarChart3, Calendar, Clock, Monitor } from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { commands } from "@/bindings"
import type {
  UiAppUsage,
  UiAppUsageCell,
  UiTimeDelta,
  UiTimeOption,
  UiWindowUsage,
  UiWindowUsageCell,
} from "@/bindings"
import { throwOnError } from '@/lib/throwOnError'

export const Route = createFileRoute('/timeline')({
  component: RouteComponent,
})

type ViewMode = "application" | "window"
type TimeUnit = "minutes" | "hours" | "days" | "weeks"

type TimelineQueryResult =
  | { type: "application"; data: Array<UiAppUsageCell> }
  | { type: "window"; data: Array<UiWindowUsageCell> }

// Map interval size options to duration + UiTimeOption
const INTERVAL_OPTIONS: Array<{ value: string; label: string; duration: number; timeOption: UiTimeOption }> = [
  { value: "15min", label: "15 minutes", duration: 15, timeOption: "Minutes" },
  { value: "30min", label: "30 minutes", duration: 30, timeOption: "Minutes" },
  { value: "1hour", label: "1 hour", duration: 1, timeOption: "Hours" },
  { value: "2hours", label: "2 hours", duration: 2, timeOption: "Hours" },
  { value: "1day", label: "1 day", duration: 1, timeOption: "Days" },
]

export default function RouteComponent() {
  const [viewMode, setViewMode] = useState<ViewMode>("application")
  const [timeUnit, setTimeUnit] = useState<TimeUnit>("hours")
  const [intervalValue, setIntervalValue] = useState<string>("1hour")
  const [expandedSlots, setExpandedSlots] = useState<Set<string>>(new Set())

  const interval = useMemo(() => INTERVAL_OPTIONS.find(o => o.value === intervalValue) ?? INTERVAL_OPTIONS[2], [intervalValue])

  const toggleSlotExpansion = (slotId: string) => {
    const newExpanded = new Set(expandedSlots)
    if (newExpanded.has(slotId)) {
      newExpanded.delete(slotId)
    } else {
      newExpanded.add(slotId)
    }
    setExpandedSlots(newExpanded)
  }

  const formatDuration = (value: UiTimeDelta | string | undefined) => {
    if (!value) return "0 min"
    if (typeof value === "string") return value

    const totalSeconds = value.secs + value.nanos / 1_000_000_000
    const totalMinutes = totalSeconds / 60

    switch (timeUnit) {
      case "hours": {
        const hours = Math.floor(totalMinutes / 60)
        const minutes = Math.floor(totalMinutes % 60)
        if (hours > 0 && minutes > 0) return `${hours}h ${minutes}m`
        if (hours > 0) return `${hours}h`
        return `${Math.floor(totalMinutes)} min`
      }
      case "days": {
        const days = Math.floor(totalMinutes / (60 * 24))
        const remainingHours = Math.floor((totalMinutes % (60 * 24)) / 60)
        if (days > 0 && remainingHours > 0) return `${days}d ${remainingHours}h`
        if (days > 0) return `${days}d`
        if (remainingHours > 0) return `${remainingHours}h`
        return `${Math.floor(totalMinutes)} min`
      }
      case "weeks": {
        const weeks = Math.floor(totalMinutes / (60 * 24 * 7))
        const remainingDays = Math.floor((totalMinutes % (60 * 24 * 7)) / (60 * 24))
        if (weeks > 0 && remainingDays > 0) return `${weeks}w ${remainingDays}d`
        if (weeks > 0) return `${weeks}w`
        if (remainingDays > 0) return `${remainingDays}d`
        return `${Math.floor(totalMinutes)} min`
      }
      default:
        return `${Math.floor(totalMinutes)} min`
    }
  }

  // Compute start/end (today → now) in UTC ISO strings expected by backend
  const now = new Date()
  const startOfTodayLocal = new Date(now)
  startOfTodayLocal.setHours(0, 0, 0, 0)
  const startUtcIso = new Date(startOfTodayLocal.getTime() - startOfTodayLocal.getTimezoneOffset() * 60000).toISOString()
  const endUtcIso = new Date(now.getTime() - now.getTimezoneOffset() * 60000).toISOString()

  const queryKey = ["timeline", viewMode, interval.value]
  const queryFn = async (): Promise<TimelineQueryResult> => {
    const params = {
      start: startUtcIso,
      end: endUtcIso,
      duration: interval.duration,
      time_option: interval.timeOption,
    } as const
    return viewMode === "application"
      ? {
        type: 'application' as const,
        data: await throwOnError(commands.getTimelineAppData(params)),
      }
      : {
        type: 'window' as const,
        data: await throwOnError(commands.getTimelineWindowData(params)),
      }
  }

  const { data: buckets, isLoading, refetch, isFetching } = useQuery<TimelineQueryResult>({ queryKey, queryFn })

  const getSlotPeriod = (startIso: string) => {
    const start = new Date(startIso)
    const end = new Date(start.getTime() + interval.duration * (
      interval.timeOption === 'Minutes' ? 60_000 :
      interval.timeOption === 'Hours' ? 3_600_000 :
      interval.timeOption === 'Days' ? 86_400_000 :
      interval.timeOption === 'Weeks' ? 7 * 86_400_000 :
      60_000
    ))
    return `${start.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })} - ${end.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`
  }

  const renderApplicationBucket = (bucket: UiAppUsageCell) => {
    const startIso = bucket.start_time
    const payload = bucket.data
    const isExpanded = expandedSlots.has(startIso)
    const items: Array<UiAppUsage> = payload?.usage ?? []
    const totalDuration = payload?.duration
    const period = getSlotPeriod(startIso)

    return (
      <Card key={startIso} className="overflow-hidden">
        <CardHeader
          className="cursor-pointer hover:bg-muted/50 transition-colors"
          onClick={() => toggleSlotExpansion(startIso)}
        >
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <CardTitle className="text-lg">{period}</CardTitle>
              {totalDuration && (
                <Badge variant="outline" className="text-xs">
                  Total: {formatDuration(totalDuration)}
                </Badge>
              )}
            </div>
          </div>
        </CardHeader>

        {isExpanded && (
          <CardContent className="pt-0">
            <div className="space-y-3">
              {items.map((item, idx) => (
                <div key={idx} className="flex items-center justify-between p-3 rounded-lg bg-muted/30">
                  <div className="flex items-center gap-3">
                    <div>
                      <p className="font-medium">{item.app_identifier}</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-3">
                    <div className="text-right">
                      <p className="font-medium">{formatDuration(item.duration)}</p>
                      <p className="text-xs text-muted-foreground capitalize">application</p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        )}
      </Card>
    )
  }

  const renderWindowBucket = (bucket: UiWindowUsageCell) => {
    const startIso = bucket.start_time
    const payload = bucket.data
    const isExpanded = expandedSlots.has(startIso)
    const items: Array<UiWindowUsage> = payload?.usage ?? []
    const totalDuration = payload?.duration
    const period = getSlotPeriod(startIso)

    return (
      <Card key={startIso} className="overflow-hidden">
        <CardHeader
          className="cursor-pointer hover:bg-muted/50 transition-colors"
          onClick={() => toggleSlotExpansion(startIso)}
        >
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <CardTitle className="text-lg">{period}</CardTitle>
              {totalDuration && (
                <Badge variant="outline" className="text-xs">
                  Total: {formatDuration(totalDuration)}
                </Badge>
              )}
            </div>
          </div>
        </CardHeader>

        {isExpanded && (
          <CardContent className="pt-0">
            <div className="space-y-3">
              {items.map((item, idx) => (
                <div key={idx} className="flex items-center justify-between p-3 rounded-lg bg-muted/30">
                  <div className="flex items-center gap-3">
                    <div>
                      <p className="font-medium">{item.window_name}</p>
                      <p className="text-sm text-muted-foreground">{item.app_identifier}</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-3">
                    <div className="text-right">
                      <p className="font-medium">{formatDuration(item.duration)}</p>
                      <p className="text-xs text-muted-foreground capitalize">window</p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        )}
      </Card>
    )
  }

  const renderTimelineContent = () => {
    if (isLoading) {
      return <Card><CardHeader><CardTitle>Loading…</CardTitle></CardHeader></Card>
    }

    if (!buckets || buckets.data.length === 0) {
      return (
        <Card>
          <CardHeader>
            <CardTitle>No activity yet</CardTitle>
          </CardHeader>
          <CardContent>
            <Button onClick={() => refetch()}>Try again</Button>
          </CardContent>
        </Card>
      )
    }

    if (buckets.type === 'application') {
      return buckets.data.map(renderApplicationBucket)
    }

    return buckets.data.map(renderWindowBucket)
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b bg-card">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <BarChart3 className="h-8 w-8 text-primary" />
              <h1 className="text-2xl font-bold text-foreground">Activity Tracker</h1>
            </div>

            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <Calendar className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm text-muted-foreground">Today</span>
              </div>
              <Button onClick={() => refetch()} disabled={isFetching} size="sm">
                {'Refresh'}
              </Button>
            </div>
          </div>
        </div>
      </header>

      <div className="container mx-auto px-4 py-6">
        {/* Controls */}
        <div className="mb-6 flex flex-wrap items-center gap-4">
          <div className="flex items-center gap-2">
            <Monitor className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm font-medium">View:</span>
            <div className="flex rounded-lg border bg-muted p-1">
              <Button
                variant={viewMode === "application" ? "default" : "ghost"}
                size="sm"
                onClick={() => setViewMode("application")}
                className="h-8"
              >
                Applications
              </Button>
              <Button
                variant={viewMode === "window" ? "default" : "ghost"}
                size="sm"
                onClick={() => setViewMode("window")}
                className="h-8"
              >
                Windows
              </Button>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Clock className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm font-medium">Interval:</span>
            <Select value={intervalValue} onValueChange={(value) => setIntervalValue(value)}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {INTERVAL_OPTIONS.map(o => (
                  <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">Unit:</span>
            <Select value={timeUnit} onValueChange={(value: TimeUnit) => setTimeUnit(value)}>
              <SelectTrigger className="w-24">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="minutes">Minutes</SelectItem>
                <SelectItem value="hours">Hours</SelectItem>
                <SelectItem value="days">Days</SelectItem>
                <SelectItem value="weeks">Weeks</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* Timeline */}
        <div className="space-y-4">
          {renderTimelineContent()}
        </div>
      </div>
    </div>
  )
}
