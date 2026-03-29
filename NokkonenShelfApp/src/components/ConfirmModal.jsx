export default function ConfirmModal({ title, message, confirmLabel = 'Delete', onConfirm, onCancel }) {
  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={e => e.stopPropagation()}>
        <div className="modal__header modal__header--danger">
          <h2>{title}</h2>
        </div>
        <div className="modal__form">
          <p className="confirm-modal__message">{message}</p>
          <div className="modal__actions">
            <button className="btn btn--ghost" onClick={onCancel}>Cancel</button>
            <button className="btn btn--danger" onClick={onConfirm}>{confirmLabel}</button>
          </div>
        </div>
      </div>
    </div>
  )
}
