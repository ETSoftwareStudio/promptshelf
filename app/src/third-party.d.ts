// The list that vite.config.ts makes with scripts/third-party.mjs.
declare module "virtual:third-party" {
  export interface ThirdParty {
    name: string;
    version?: string;
    licence: string;
    /** Only where the licence asks for it (MPL-2.0), and for the font. */
    source?: string;
  }
  const list: { crates: ThirdParty[]; npm: ThirdParty[]; fonts: ThirdParty[]; error?: string };
  export default list;
}
