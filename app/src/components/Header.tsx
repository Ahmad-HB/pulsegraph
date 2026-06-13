export function Header({ today, avatar, onAvatarClick }: { today: string; avatar?: string | null; onAvatarClick?: () => void; }) {
  return (
    <div className="header">
      <div className="header-top">
        <div className="title-wrap">
          <button className="avatar-btn" onClick={onAvatarClick} title="Set profile image" aria-label="Set profile image">
            {avatar ? <img className="avatar" src={avatar} alt="" /> : <span className="avatar placeholder">+</span>}
          </button>
          <div className="title">PulseGraph <small>· Today</small></div>
        </div>
        <div className="big">{today}</div>
      </div>
    </div>
  );
}
