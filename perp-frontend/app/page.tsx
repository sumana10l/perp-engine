import OpenPositionForm from "../components/OpenPositionForm";
import PositionsTable from "../components/PositionsTable";

export default function Home() {
  return (
    <main className="min-h-screen p-10 bg-gray-100 text-black">
      <h1 className="text-3xl font-bold mb-10">
        Perp Trading Dashboard
      </h1>

      <div className="grid grid-cols-2 gap-12 items-start">
        <OpenPositionForm />
        <PositionsTable />
      </div>
    </main>
  );
}