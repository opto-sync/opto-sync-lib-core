// Independently authored parity fixture for this repository's CI consumer gate.
// This generic fixture does not claim unrelated Opto Sync contracts have converged.
// JSON Schema B generated from this file is comparison evidence only.
import "@typespec/json-schema";
using TypeSpec.JsonSchema;

namespace OreSchemaAuthority;

@id("AuthorityKind")
enum AuthorityKind {
  typespec: "typespec",
  jsonSchema: "json-schema",
}

@id("PeerAuthorityRegistration")
model PeerAuthorityRegistration {
  repository: string;
  authority: AuthorityKind;
  generatedWitnessOnly: boolean;
  owner?: string;
}
