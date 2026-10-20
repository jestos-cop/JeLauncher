import { useTranslation } from 'react-i18next'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'

interface RelatedChangesDialogProps {
  title: string
  description: string
  /** Display names of the other files the change touches. */
  names: string[]
  withRelatedLabel: string
  onlySelectedLabel: string
  onWithRelated: () => void
  onOnlySelected: () => void
  onClose: () => void
}

/** Asks whether a change to one file should also apply to the files linked to it. */
export function RelatedChangesDialog({
  title,
  description,
  names,
  withRelatedLabel,
  onlySelectedLabel,
  onWithRelated,
  onOnlySelected,
  onClose,
}: RelatedChangesDialogProps) {
  const { t } = useTranslation()
  return (
    <AlertDialog open onOpenChange={(open) => !open && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription>{description}</AlertDialogDescription>
        </AlertDialogHeader>
        <ul className="max-h-48 list-disc overflow-auto pl-5 text-sm">
          {names.map((name) => (
            <li key={name}>{name}</li>
          ))}
        </ul>
        <AlertDialogFooter>
          <AlertDialogCancel>{t('action.cancel')}</AlertDialogCancel>
          <Button variant="outline" onClick={onOnlySelected}>
            {onlySelectedLabel}
          </Button>
          <Button onClick={onWithRelated}>{withRelatedLabel}</Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
