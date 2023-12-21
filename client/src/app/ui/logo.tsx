import { BuildingStorefrontIcon } from '@heroicons/react/24/outline';
import { lusitana } from '@/app/ui/fonts';

export default function Logo() {
  return (
    <div
      className={`${lusitana.className} flex flex-row items-center leading-none text-white`}
    >
      <BuildingStorefrontIcon className="h-12 w-12" />
      <p className="text-[44px]">My Hood</p>
    </div>
  );
}
