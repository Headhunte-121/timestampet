import Library from '../src/components/Library';

export default function TestApp() {
  return (
    <div className="w-full min-h-screen bg-[#0D0F14] text-white font-sans">
      <Library type="Movie" onMediaSelect={() => {}} />
    </div>
  );
}
