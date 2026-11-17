# Synthetic host-normalized conformance input, not a Cairn producer receipt.
# These fixture identities confer no authority or live implementation evidence.
{
  intent = {
    schema = "lattice.workflow-intent.v1",
    cohort = {
      producer_identity = "b3:f5b75e2e9a5c980783a64efeabfdfb4eb84695ae81ecdcb3fc13e245b4b26319",
      selection_identity = "b3:ae99985a7cc64bb3887e314a99fab24c24893ba84706b63d33631a1b727fdbfc",
      obligations_identity = "b3:c76c3a8d2a7b5e29fd6281827a4c3d89dbf7fb695b2cd0d860d55241b3d3fc23",
      profile_identity = "b3:cfa4727331dc296a7b57c8b7ac819e6e8244f510ab1cc1122fbd90df23fc2ade",
      context_identity = "b3:437da932076aac665e73e06f1cde178e7fe46b8ceaa5d6b25be344244c6c4388",
      source_identity = "b3:035e4fc7814ee88e9f5f3ead713108c27a70caee860f2da80820cf3454616c91",
      policy_identity = "b3:653de33e5515c82a8182a442f60738f41f5336fb8c6388ca027797614d6cf8d2",
      dependency_identity = "b3:b6ab6132ba854d2e79eef3373c06ba6d69aa8a8edd21c8c92fbb79dd3c4ea415",
    },
    coverage = "complete",
    unsupported_fields = [],
    obligations = [
      {
        id = "A",
        obligation_identity = "b3:d2ad7a311ae8eb940f0016e90da928d6ad36f2081ff263325c4666ea9cc45939",
        guidance_identity = "b3:fa1f55609ce3d7099fd09266842c46978628a9887023030642b52b792b68771d",
        remaining_condition_identity = "b3:5a656b93fbad4e19417e9b0f053bcf45fc3b62c3259628b165047776140a8392",
        operation_class = "cairn_validate",
        action_identity = "b3:1dfd814a0206a313a753a6b8b57a94f6c8fd7d376c31e319f8d99912e9464c8a",
        prerequisites = [],
        required_observations = [
          {
            id = "evidence-A",
            schema = "lattice.workflow-intent-fixture-result.v1",
            predicate_identity = "b3:46abd827734dda92680241a1a42afac1157a620f783a63addd6422c164465c59",
          },
        ],
        precondition_identity = "b3:8d99e4a0901d061cc9bb8d176d60d450f6b3787c2cc50d29bdce779f57124553",
        required_capabilities = [
          "native-cairn",
        ],
      },
      {
        id = "B",
        obligation_identity = "b3:1e8b18b4415ffe488198fdf9749c4ce2f881af8ab0bb38fd8d437fff9d938d7d",
        guidance_identity = "b3:d2ce07f30deb51cc61fa60c2dfbe4c85a02cc4e39335ad33fae32dfc45f06604",
        remaining_condition_identity = "b3:55235909d9794ddce0c3286924790ffc3460262e4f808f9e6d7bd5c03436e470",
        operation_class = "review_wait",
        action_identity = "b3:381bb2acfd7c3aa9645b3a1886a80a9877ed1ba0a442b484027d876391b7dae8",
        prerequisites = [
          "A",
        ],
        required_observations = [
          {
            id = "evidence-B",
            schema = "lattice.workflow-intent-fixture-result.v1",
            predicate_identity = "b3:4205a9a8e8eb68ae86363a1714f6cab99ed3d23b31cd0fef48b14b701ecb39c4",
          },
        ],
        precondition_identity = "b3:ff2ec9b496375a4c3fd29399a11eccdd7afe9a34e65ae16939a6603265fd44bf",
        required_capabilities = [
          "native-cairn",
        ],
      },
      {
        id = "C",
        obligation_identity = "b3:864233d10128c076489d8617405f9b6495c40eaebd18bd729833df6ea743a320",
        guidance_identity = "b3:4962c891d87de1acd6a166f6f405c37f6d3b04cb1c693a5ecd89efb0f41c273a",
        remaining_condition_identity = "b3:87551892b696a7a4045f902b9155f7e7ac398118e1ee3b50b6d7f5f1b25de71e",
        operation_class = "named_check",
        action_identity = "b3:75575e56846313b1c30340dc89ea51a0bd54734a571c7b65bd086fe268292df9",
        prerequisites = [],
        required_observations = [
          {
            id = "evidence-C",
            schema = "lattice.workflow-intent-fixture-result.v1",
            predicate_identity = "b3:64443b9373804764a91578f788fbca34f0cf8d1e8f08d31a77036239d58c4254",
          },
        ],
        precondition_identity = "b3:c746a6bd97f8868747ebdc21476363d24223a5b0be94fe6fdbea8e6ec6e5d956",
        required_capabilities = [
          "native-cairn",
        ],
      },
      {
        id = "D",
        obligation_identity = "b3:43668efc7484735d0222c13b84e827c530c0016c472511e2864f6a8ca31c2022",
        guidance_identity = "b3:3e55a33995c6b5f473592b4efe091c6a926cfbd284e38d22d636bda29da49a97",
        remaining_condition_identity = "b3:7ef8485f82b0861e4d569f420005da2f765266bae9d9362ede9892679afe6545",
        operation_class = "completion_only",
        action_identity = "b3:ab33f46013a21d5571f302d96123afdace4480e0b328f27a16a830df992c4749",
        prerequisites = [
          "B",
          "C",
        ],
        required_observations = [
          {
            id = "evidence-D",
            schema = "lattice.workflow-intent-fixture-result.v1",
            predicate_identity = "b3:e69bfcc4e22f09e854142e8f1e6b2e1a2f8f803dfcf59585dbf6b12e6de1047c",
          },
        ],
        precondition_identity = "b3:abcd798865f72d1dd01f1b2855eac150b0cf48971e81fd70156f11af993fef06",
        required_capabilities = [
          "native-cairn",
        ],
      },
    ],
    alternatives = [],
  },
  facts = {
    expected_cohort = {
      producer_identity = "b3:f5b75e2e9a5c980783a64efeabfdfb4eb84695ae81ecdcb3fc13e245b4b26319",
      selection_identity = "b3:ae99985a7cc64bb3887e314a99fab24c24893ba84706b63d33631a1b727fdbfc",
      obligations_identity = "b3:c76c3a8d2a7b5e29fd6281827a4c3d89dbf7fb695b2cd0d860d55241b3d3fc23",
      profile_identity = "b3:cfa4727331dc296a7b57c8b7ac819e6e8244f510ab1cc1122fbd90df23fc2ade",
      context_identity = "b3:437da932076aac665e73e06f1cde178e7fe46b8ceaa5d6b25be344244c6c4388",
      source_identity = "b3:035e4fc7814ee88e9f5f3ead713108c27a70caee860f2da80820cf3454616c91",
      policy_identity = "b3:653de33e5515c82a8182a442f60738f41f5336fb8c6388ca027797614d6cf8d2",
      dependency_identity = "b3:b6ab6132ba854d2e79eef3373c06ba6d69aa8a8edd21c8c92fbb79dd3c4ea415",
    },
    compiler_artifact_identity = "b3:7ead1042c0ea42ce4a48bf8b631b04c3585421dd91175df834c7ae6ec08f6dd7",
    runtime_profile_identity = "b3:6e6a910cf45978787ac019beff79ac82eb5a0f0a1a3f7ff2071c97cfc3bce3f7",
    available_capabilities = [
      "native-cairn",
    ],
    available_implementations = [],
  },
}
