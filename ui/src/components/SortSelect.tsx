import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { commands, type Provider, type SortOrder } from '@/bindings'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

interface SortSelectProps {
  provider: Provider
  value: SortOrder
  onChange: (sort: SortOrder) => void
}

/** Search order picker with the orders the platform supports; hidden when there is no choice. */
export function SortSelect({ provider, value, onChange }: SortSelectProps) {
  const { t } = useTranslation()
  const { data: orders = [] } = useQuery({
    queryKey: ['sort-orders', provider],
    queryFn: () => commands.listSortOrders(provider),
    staleTime: Infinity,
  })
  if (orders.length < 2) return null
  return (
    <Select value={value} onValueChange={(sort) => onChange(sort as SortOrder)}>
      <SelectTrigger className="w-44">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {orders.map((order) => (
          <SelectItem key={order} value={order}>
            {t(`sort.${order}`)}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
